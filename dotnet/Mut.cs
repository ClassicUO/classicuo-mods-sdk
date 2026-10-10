namespace CuoModSdk;

/// <summary>
/// A writable query term: <c>Query&lt;Data&lt;Mut&lt;Node&gt;, Text&gt;&gt;</c>. The row
/// carries a handle into SDK-owned row storage; change the component through
/// <see cref="Value"/> and it is written back when the system (or observer) returns —
/// after the run's own commands, so it wins over a <c>Commands.Insert</c> of the same
/// component (<c>query.set</c>, or <c>commands.insert</c> when the run has a Commands) — and ONLY when the value actually differs from what the host returned (an
/// unconditional write would fire <c>Changed</c> and UI relayout every frame).
///
/// <para>The handle is a (storage, row) pair: copying it is fine, every copy names the
/// same value. Touching <see cref="Value"/> marks the row for the compare; a row never
/// touched costs nothing at write-back. A read-only host type (<c>cuo:player/hits</c>,
/// <c>cuo:ui/computed</c>, …) as a <c>Mut</c> term fails the mod's load.</para>
/// </summary>
public readonly struct Mut<T> : IMutTerm
{
    readonly MutColumn<T> _column;
    readonly int _row;

    internal Mut(MutColumn<T> column, int row)
    {
        _column = column;
        _row = row;
    }

    /// <summary>The component, by reference. Any access marks the row for write-back.</summary>
    public ref T Value => ref _column.Touch(_row);

    object IMutTerm.Binder => new MutBinder();

    sealed class MutBinder : MutBinder<Mut<T>>
    {
        internal override void Describe(QueryTermSink sink) => sink.Mut<T>();

        internal override Mut<T> Bind(RowReader row, int index)
        {
            var key = (row.Slot << 8) | index;
            var columns = row.Scope.MutColumns;
            if (!columns.TryGetValue(key, out var column))
                columns[key] = column = new MutColumn<T>(row.Scope, row.Slot, row.Rows, index);
            return new Mut<T>((MutColumn<T>)column, row.RowIndex);
        }
    }
}

// The bridge from a Data<> slot's type argument to the Mut handling, without reflection:
// `default(T) is IMutTerm` is true only for a Mut<X>, and it hands out a binder typed on
// the SLOT type, so Data<T1..> never needs to know X.
internal interface IMutTerm
{
    object Binder { get; }
}

internal abstract class MutBinder<TTerm>
{
    internal abstract void Describe(QueryTermSink sink);

    internal abstract TTerm Bind(RowReader row, int index);
}

internal static class MutTerm<T>
{
    internal static readonly MutBinder<T>? Binder = default(T) is IMutTerm m ? (MutBinder<T>)m.Binder : null;
}

/// <summary>
/// One Mut term's values for one run: parsed lazily per row (so a TryGet and a foreach
/// over the same row share a slot), compared and written back after the run.
/// </summary>
internal sealed unsafe class MutColumn<T>
{
    readonly RunScope _scope;
    readonly int _slot;
    readonly nint _rows;
    readonly int _index;
    // A typed query: values lift from (and compare against) the typed column; the write is set-<x>.
    readonly TypedColumn<T>? _typed;
    readonly T[] _values;
    readonly bool[] _loaded;
    readonly bool[] _dirty;

    internal MutColumn(RunScope scope, int slot, nint rows, int index)
    {
        _scope = scope;
        _slot = slot;
        _rows = rows;
        _index = index;
        var count = scope.Rows(slot).Count;
        if (scope.IsTyped(slot))
            _typed = scope.Column(slot, index, Typed<T>.Codec!);
        _values = new T[count];
        _loaded = new bool[count];
        _dirty = new bool[count];
        scope.AfterRun(WriteBack);
    }

    internal ref T Touch(int row)
    {
        if (!_loaded[row])
        {
            _loaded[row] = true;
            _values[row] = Parse(row);
        }
        _dirty[row] = true;
        return ref _values[row];
    }

    T Parse(int row) =>
        _typed != null ? _typed.Get(_scope, _slot, row) : RowReader.ReadComp<T>(_scope, _rows, row, _index);

    void WriteBack()
    {
        if (_typed != null)
        {
            WriteBackTyped(_typed);
            return;
        }
        var info = _scope.Host.Json<T>();
        string? path = null;
        for (var i = 0; i < _dirty.Length; i++)
        {
            if (!_dirty[i])
                continue;
            // Compare against our own serialization of the host's value, not the host's
            // bytes: the two format differently, and a spurious write marks it changed.
            var original = JsonScratch.Rent(Parse(i), info);
            try
            {
                if (!JsonScratch.Differs(_values[i], info, original.Span, out var now))
                    continue;
                var entity = EcsAbi.RowEntity(_rows, i, EcsAbi.RowStride);
                // With a Commands param in the run, write through commands.insert: it is
                // applied after the run's own commands (query.set lands at once, so a
                // Commands.Insert of the same component would win over the Mut).
                if (_scope.CommandsHandle != 0)
                    _scope.Commands.InsertJson(entity, path ??= ModHost.PathOf<T>(), now);
                else
                    EcsAbi.QuerySet(_scope.Handle(_slot), entity, (byte)_index, now);
            }
            finally
            {
                original.Return();
            }
        }
    }

    // query.set is deferred in order with the run's commands on every host, so the write
    // lands after this run's own Commands.Insert of the same component either way.
    void WriteBackTyped(TypedColumn<T> column)
    {
        var codec = column.Codec;
        var handle = _scope.Handle(_slot);
        var rec = stackalloc byte[codec.Size];
        for (var i = 0; i < _dirty.Length; i++)
        {
            if (!_dirty[i] || !codec.Differs(_values[i], column.Element(_scope, _slot, i)))
                continue;
            codec.LowerInto(_values[i], rec);
            codec.Set(handle, EcsAbi.RowEntity(_rows, i, EcsAbi.EntityStride), (byte)_index, rec);
        }
    }
}
