namespace CuoModSdk;

/// <summary>
/// A writable query term: <c>Query&lt;Data&lt;Mut&lt;Node&gt;, Text&gt;&gt;</c>. The row
/// carries a handle into SDK-owned row storage; change the component through
/// <see cref="Value"/> and it is written back (<c>set-&lt;x&gt;</c>) when the system (or
/// observer) returns — after the run's own commands, so it wins over a
/// <c>Commands.Insert</c> of the same component — and ONLY when the value actually differs
/// from what the host returned (an unconditional write would fire <c>Changed</c> and UI
/// relayout every frame).
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
                columns[key] = column = new MutColumn<T>(row.Scope, row.Slot, row.Entities, index);
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
/// One Mut term's values for one run: lifted lazily per row from the term's column (so a
/// TryGet and a foreach over the same row share a slot), compared and written back after
/// the run.
/// </summary>
internal sealed unsafe class MutColumn<T>
{
    readonly RunScope _scope;
    readonly int _slot;
    readonly nint _entities;
    readonly int _index;
    readonly TypedColumn<T> _column;
    readonly T[] _values;
    readonly bool[] _loaded;
    readonly bool[] _dirty;

    internal MutColumn(RunScope scope, int slot, nint entities, int index)
    {
        _scope = scope;
        _slot = slot;
        _entities = entities;
        _index = index;
        var count = scope.Entities(slot).Count;
        _column = scope.Column(slot, index, Typed<T>.Record);
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
            _values[row] = _column.Get(_scope, _slot, row);
        }
        _dirty[row] = true;
        return ref _values[row];
    }

    // set-<x> is deferred in order with the run's commands, so the write lands after this
    // run's own Commands.Insert of the same component.
    void WriteBack()
    {
        var codec = _column.Codec;
        var handle = _scope.Handle(_slot);
        var rec = stackalloc byte[codec.Size];
        for (var i = 0; i < _dirty.Length; i++)
        {
            if (!_dirty[i] || !codec.Differs(_values[i], _column.Element(_scope, _slot, i)))
                continue;
            codec.LowerInto(_values[i], rec);
            codec.Set(handle, EcsAbi.EntityAt(_entities, i), (byte)_index, rec);
        }
    }
}
