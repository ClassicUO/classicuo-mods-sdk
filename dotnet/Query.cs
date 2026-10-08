using ModAbi;

namespace CuoModSdk;

/// <summary>
/// The read side of a query: which components the row carries, and how to rebuild them
/// from a pushed row. Implemented by <see cref="Data{T1}"/> and friends.
/// </summary>
public interface IQueryData<TSelf> where TSelf : struct, IQueryData<TSelf>
{
    /// <summary>Append the read terms, in declaration order.</summary>
    static abstract void Describe(QueryTermSink sink);

    static abstract TSelf Read(RowReader row);
}

/// <summary>The filter side of a query. Implemented by <see cref="With{T}"/> / <see cref="Without{T}"/> / <see cref="Changed{T}"/> / <see cref="Added{T}"/> / <see cref="Filter{F1}"/>.</summary>
public interface IQueryFilter<TSelf> where TSelf : struct, IQueryFilter<TSelf>
{
    static abstract void Describe(QueryTermSink sink);
}

/// <summary>
/// The terms of one query, as its <c>TData</c> then its <c>TFilter</c> describe
/// themselves. Reading terms (<c>Ref</c> / <c>Changed</c> / <c>Added</c>) carry the
/// component into the row in declaration order — which is why <c>TData</c> always
/// describes FIRST, so <c>Data&lt;T1..Tn&gt;</c> owns row indices 0..n-1 no matter what
/// the filter appends.
/// </summary>
public sealed class QueryTermSink
{
    readonly ModHost _host;
    internal readonly List<QueryTermT> Terms = new();

    internal QueryTermSink(ModHost host) => _host = host;

    /// <summary>A read term: the row carries <typeparamref name="T"/> (a <see cref="Mut{T}"/> declares a writable term).</summary>
    internal void Read<T>()
    {
        if (MutTerm<T>.Binder is { } mut)
            mut.Describe(this);
        else
            Terms.Add(new QueryTermT { Kind = QueryTermKind.Ref, TypeId = _host.Id<T>() });
    }

    internal void Mut<T>() => Terms.Add(new QueryTermT { Kind = QueryTermKind.Mut, TypeId = _host.Id<T>() });

    /// <summary>Presence filter. Dropped when the type is already a term — a read term already implies presence.</summary>
    internal void With<T>()
    {
        var id = _host.Id<T>();
        if (Find(id) < 0)
            Terms.Add(new QueryTermT { Kind = QueryTermKind.With, TypeId = id });
    }

    internal void Without<T>() => Terms.Add(new QueryTermT { Kind = QueryTermKind.Without, TypeId = _host.Id<T>() });

    internal void Changed<T>() => Reading<T>(QueryTermKind.Changed);

    internal void Added<T>() => Reading<T>(QueryTermKind.Added);

    // Read + change / added filter. When the type is already a term (a Data<T> read, or
    // a redundant With<T>) the kind is upgraded IN PLACE: two terms on one type would
    // put two payloads in the row and shift every later read index. A Mut term keeps
    // its kind (upgrading would drop the write) and the filter rides as its own term —
    // appended after Data's, so its payload lands past every Data read index.
    void Reading<T>(QueryTermKind kind)
    {
        var id = _host.Id<T>();
        var at = Find(id);
        if (at >= 0 && Terms[at].Kind != QueryTermKind.Mut)
            Terms[at].Kind = kind;
        else
            Terms.Add(new QueryTermT { Kind = kind, TypeId = id });
    }

    int Find(ushort typeId)
    {
        for (var i = 0; i < Terms.Count; i++)
            if (Terms[i].TypeId == typeId && Terms[i].Kind != QueryTermKind.Without)
                return i;
        return -1;
    }
}

/// <summary>One pushed row, being turned back into a <c>Data&lt;…&gt;</c>.</summary>
public readonly struct RowReader
{
    internal readonly RunScope Scope;
    internal readonly QueryRowsView Rows;
    internal readonly int Slot;
    internal readonly int RowIndex;
    readonly RowView _row;

    internal RowReader(RunScope scope, int slot, QueryRowsView rows, int rowIndex)
    {
        Scope = scope;
        Slot = slot;
        Rows = rows;
        RowIndex = rowIndex;
        _row = rows.Row(rowIndex);
    }

    public Entity Entity => new(_row.Entity);

    /// <summary>The component at read-term index <paramref name="index"/> (<c>default</c> when the slot is empty).</summary>
    public T Comp<T>(int index)
    {
        if (MutTerm<T>.Binder is { } mut)
            return mut.Bind(this, index);
        return _row.Comp(index) is { } comp ? comp.Parse(Scope.Host.Json<T>())! : default!;
    }
}

// ── Data ─────────────────────────────────────────────────────────────────────────

/// <summary>An entity id and nothing else — for a filter-only query.</summary>
public struct Data : IQueryData<Data>
{
    public Entity Entity;

    public static void Describe(QueryTermSink sink) { }

    public static Data Read(RowReader row) => new() { Entity = row.Entity };

    public static implicit operator Entity(Data row) => row.Entity;
}

/// <summary>The row carries <typeparamref name="T1"/>. Deconstructs as <c>(entity, t1)</c>.</summary>
public struct Data<T1> : IQueryData<Data<T1>>
{
    public Entity Entity;
    public T1 Item1;

    public void Deconstruct(out Entity entity, out T1 item1)
    {
        entity = Entity;
        item1 = Item1;
    }

    public static void Describe(QueryTermSink sink) => sink.Read<T1>();

    public static Data<T1> Read(RowReader row) => new() { Entity = row.Entity, Item1 = row.Comp<T1>(0) };
}

/// <summary>The row carries <typeparamref name="T1"/> and <typeparamref name="T2"/>. Deconstructs as <c>(entity, t1, t2)</c>.</summary>
public struct Data<T1, T2> : IQueryData<Data<T1, T2>>
{
    public Entity Entity;
    public T1 Item1;
    public T2 Item2;

    public void Deconstruct(out Entity entity, out T1 item1, out T2 item2)
    {
        entity = Entity;
        item1 = Item1;
        item2 = Item2;
    }

    public static void Describe(QueryTermSink sink)
    {
        sink.Read<T1>();
        sink.Read<T2>();
    }

    public static Data<T1, T2> Read(RowReader row) =>
        new() { Entity = row.Entity, Item1 = row.Comp<T1>(0), Item2 = row.Comp<T2>(1) };
}

/// <summary>Three-component row. Deconstructs as <c>(entity, t1, t2, t3)</c>.</summary>
public struct Data<T1, T2, T3> : IQueryData<Data<T1, T2, T3>>
{
    public Entity Entity;
    public T1 Item1;
    public T2 Item2;
    public T3 Item3;

    public void Deconstruct(out Entity entity, out T1 item1, out T2 item2, out T3 item3)
    {
        entity = Entity;
        item1 = Item1;
        item2 = Item2;
        item3 = Item3;
    }

    public static void Describe(QueryTermSink sink)
    {
        sink.Read<T1>();
        sink.Read<T2>();
        sink.Read<T3>();
    }

    public static Data<T1, T2, T3> Read(RowReader row) =>
        new() { Entity = row.Entity, Item1 = row.Comp<T1>(0), Item2 = row.Comp<T2>(1), Item3 = row.Comp<T3>(2) };
}

/// <summary>Four-component row. Deconstructs as <c>(entity, t1, t2, t3, t4)</c>.</summary>
public struct Data<T1, T2, T3, T4> : IQueryData<Data<T1, T2, T3, T4>>
{
    public Entity Entity;
    public T1 Item1;
    public T2 Item2;
    public T3 Item3;
    public T4 Item4;

    public void Deconstruct(out Entity entity, out T1 item1, out T2 item2, out T3 item3, out T4 item4)
    {
        entity = Entity;
        item1 = Item1;
        item2 = Item2;
        item3 = Item3;
        item4 = Item4;
    }

    public static void Describe(QueryTermSink sink)
    {
        sink.Read<T1>();
        sink.Read<T2>();
        sink.Read<T3>();
        sink.Read<T4>();
    }

    public static Data<T1, T2, T3, T4> Read(RowReader row) =>
        new()
        {
            Entity = row.Entity,
            Item1 = row.Comp<T1>(0),
            Item2 = row.Comp<T2>(1),
            Item3 = row.Comp<T3>(2),
            Item4 = row.Comp<T4>(3),
        };
}

// ── Filters ──────────────────────────────────────────────────────────────────────

/// <summary>The entity must have <typeparamref name="T"/>; the row does not carry it.</summary>
public readonly struct With<T> : IQueryFilter<With<T>>
{
    public static void Describe(QueryTermSink sink) => sink.With<T>();
}

/// <summary>The entity must NOT have <typeparamref name="T"/>.</summary>
public readonly struct Without<T> : IQueryFilter<Without<T>>
{
    public static void Describe(QueryTermSink sink) => sink.Without<T>();
}

/// <summary>
/// Only entities whose <typeparamref name="T"/> changed since this system last ran. Both
/// a filter AND a read term on the wire: pair it with <c>Data&lt;T&gt;</c> to see the
/// value, and the SDK still emits ONE term.
/// </summary>
public readonly struct Changed<T> : IQueryFilter<Changed<T>>
{
    public static void Describe(QueryTermSink sink) => sink.Changed<T>();
}

/// <summary>Only entities that got <typeparamref name="T"/> since this system last ran (a reading term, like <see cref="Changed{T}"/>).</summary>
public readonly struct Added<T> : IQueryFilter<Added<T>>
{
    public static void Describe(QueryTermSink sink) => sink.Added<T>();
}

/// <summary>No filter — every entity carrying the <c>Data</c> components matches.</summary>
public readonly struct NoFilter : IQueryFilter<NoFilter>
{
    public static void Describe(QueryTermSink sink) { }
}

/// <summary>Filters, ANDed.</summary>
public readonly struct Filter<F1> : IQueryFilter<Filter<F1>>
    where F1 : struct, IQueryFilter<F1>
{
    public static void Describe(QueryTermSink sink) => F1.Describe(sink);
}

/// <summary>Filters, ANDed.</summary>
public readonly struct Filter<F1, F2> : IQueryFilter<Filter<F1, F2>>
    where F1 : struct, IQueryFilter<F1>
    where F2 : struct, IQueryFilter<F2>
{
    public static void Describe(QueryTermSink sink)
    {
        F1.Describe(sink);
        F2.Describe(sink);
    }
}

/// <summary>Filters, ANDed.</summary>
public readonly struct Filter<F1, F2, F3> : IQueryFilter<Filter<F1, F2, F3>>
    where F1 : struct, IQueryFilter<F1>
    where F2 : struct, IQueryFilter<F2>
    where F3 : struct, IQueryFilter<F3>
{
    public static void Describe(QueryTermSink sink)
    {
        F1.Describe(sink);
        F2.Describe(sink);
        F3.Describe(sink);
    }
}

/// <summary>Filters, ANDed.</summary>
public readonly struct Filter<F1, F2, F3, F4> : IQueryFilter<Filter<F1, F2, F3, F4>>
    where F1 : struct, IQueryFilter<F1>
    where F2 : struct, IQueryFilter<F2>
    where F3 : struct, IQueryFilter<F3>
    where F4 : struct, IQueryFilter<F4>
{
    public static void Describe(QueryTermSink sink)
    {
        F1.Describe(sink);
        F2.Describe(sink);
        F3.Describe(sink);
        F4.Describe(sink);
    }
}

// ── Query ──────────────────────────────────────────────────────────────────────

/// <summary>
/// The entities that have <c>TData</c> (and pass <c>TFilter</c>), with their
/// components, as the host pushed them for THIS run. A snapshot: the components are
/// value copies — declare a term as <see cref="Mut{T}"/> to write it back (applied
/// after the run, only when changed), or use <c>Commands.Insert</c>.
///
/// <para>Declaration order of the terms is load-bearing on the host: it picks the FIRST
/// present-required term as the scan driver, so put the narrowest one (a
/// <see cref="Changed{T}"/>, a rare marker) first.</para>
/// </summary>
public readonly struct Query<TData, TFilter> : ISystemParam<Query<TData, TFilter>>
    where TData : struct, IQueryData<TData>
    where TFilter : struct, IQueryFilter<TFilter>
{
    readonly RunScope _scope;
    readonly int _slot;
    readonly QueryRowsView _rows;

    Query(RunScope scope, int slot, QueryRowsView rows)
    {
        _scope = scope;
        _slot = slot;
        _rows = rows;
    }

    public static void Describe(ParamDescriber d)
    {
        var sink = new QueryTermSink(d.Host);
        // TData first: it owns read indices 0..n-1 (see QueryTermSink).
        TData.Describe(sink);
        TFilter.Describe(sink);
        d.Add(new ParamDeclT { Kind = ParamKind.Query, Query = new QueryDeclT { Terms = sink.Terms }, TypeId = ModHost.NoneType });
    }

    public static bool TryCreate(ParamContext c, out Query<TData, TFilter> value)
    {
        value = new Query<TData, TFilter>(c.Scope, c.Slot, c.Scope.Input.Rows(c.Slot));
        return true;
    }

    public int Count => _rows.Count;

    public bool IsEmpty => _rows.Count == 0;

    public bool Contains(Entity entity) => TryGet(entity, out _);

    /// <summary>The row for <paramref name="entity"/>; throws when it is not among this run's rows.</summary>
    public TData Get(Entity entity) =>
        TryGet(entity, out var data)
            ? data
            : throw new InvalidOperationException(
                $"{entity} is not among this query's {Count} row(s) — test with Contains/TryGet first");

    public bool TryGet(Entity entity, out TData data)
    {
        var id = entity.Id;
        for (var i = 0; i < _rows.Count; i++)
            if (_rows.Row(i).Entity == id)
            {
                data = TData.Read(new RowReader(_scope, _slot, _rows, i));
                return true;
            }
        data = default;
        return false;
    }

    /// <summary>The only row; false when there are none or several.</summary>
    public bool TrySingle(out TData data)
    {
        if (_rows.Count != 1)
        {
            data = default;
            return false;
        }
        data = TData.Read(new RowReader(_scope, _slot, _rows, 0));
        return true;
    }

    public Enumerator GetEnumerator() => new(_scope, _slot, _rows);

    /// <summary>Struct enumerator — <c>foreach</c> over a query allocates nothing.</summary>
    public struct Enumerator
    {
        readonly RunScope _scope;
        readonly int _slot;
        readonly QueryRowsView _rows;
        int _index;

        internal Enumerator(RunScope scope, int slot, QueryRowsView rows)
        {
            _scope = scope;
            _slot = slot;
            _rows = rows;
            _index = -1;
            Current = default;
        }

        public TData Current { get; private set; }

        public bool MoveNext()
        {
            if (++_index >= _rows.Count)
                return false;
            Current = TData.Read(new RowReader(_scope, _slot, _rows, _index));
            return true;
        }
    }
}

/// <summary>An unfiltered <see cref="Query{TData, TFilter}"/>.</summary>
public readonly struct Query<TData> : ISystemParam<Query<TData>>
    where TData : struct, IQueryData<TData>
{
    readonly Query<TData, NoFilter> _inner;

    Query(Query<TData, NoFilter> inner) => _inner = inner;

    public static void Describe(ParamDescriber d) => Query<TData, NoFilter>.Describe(d);

    public static bool TryCreate(ParamContext c, out Query<TData> value)
    {
        Query<TData, NoFilter>.TryCreate(c, out var inner);
        value = new Query<TData>(inner);
        return true;
    }

    public int Count => _inner.Count;

    public bool IsEmpty => _inner.IsEmpty;

    public bool Contains(Entity entity) => _inner.Contains(entity);

    public TData Get(Entity entity) => _inner.Get(entity);

    public bool TryGet(Entity entity, out TData data) => _inner.TryGet(entity, out data);

    public bool TrySingle(out TData data) => _inner.TrySingle(out data);

    public Query<TData, NoFilter>.Enumerator GetEnumerator() => _inner.GetEnumerator();
}
