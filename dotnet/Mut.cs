using System.Text.Json;

namespace CuoModSdk;

/// <summary>
/// A writable query term: <c>Query&lt;Data&lt;Mut&lt;Node&gt;, Text&gt;&gt;</c>. The row
/// carries a handle into SDK-owned row storage; change the component through
/// <see cref="Value"/> and it is written back when the system (or observer) returns —
/// after the run's own commands, so it wins over a <c>Commands.Insert</c> of the same
/// component — and ONLY when the value actually differs from what the host pushed (an
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
            var columns = row.Scope.MutColumns ??= new Dictionary<int, object>();
            if (!columns.TryGetValue(key, out var column))
                columns[key] = column = new MutColumn<T>(row.Scope, row.Rows, index);
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
internal sealed class MutColumn<T>
{
    readonly RunScope _scope;
    readonly QueryRowsView _rows;
    readonly int _index;
    readonly T[] _values;
    readonly bool[] _loaded;
    readonly bool[] _dirty;

    internal MutColumn(RunScope scope, QueryRowsView rows, int index)
    {
        _scope = scope;
        _rows = rows;
        _index = index;
        _values = new T[rows.Count];
        _loaded = new bool[rows.Count];
        _dirty = new bool[rows.Count];
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

    T Parse(int row) => _rows.Row(row).Comp(_index) is { } comp ? comp.Parse(_scope.Host.Json<T>())! : default!;

    void WriteBack()
    {
        var info = _scope.Host.Json<T>();
        for (var i = 0; i < _dirty.Length; i++)
        {
            if (!_dirty[i])
                continue;
            // Compare against our own serialization of the pushed value, not the host's
            // bytes: the two format differently, and a spurious write marks it changed.
            // Both sides go through the reused scratch; only a change allocates a payload.
            var original = JsonScratch.Rent(Parse(i), info);
            try
            {
                if (JsonScratch.Differs(_values[i], info, original.Span, out var now))
                    _scope.Commands.InsertRaw(new Entity(_rows.Row(i).Entity), Comp.FromBytes(_scope.Host.Id<T>(), now));
            }
            finally
            {
                original.Return();
            }
        }
    }
}
