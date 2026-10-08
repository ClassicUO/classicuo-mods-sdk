using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using ModAbi;

namespace CuoModSdk;

/// <summary>
/// The parameter data the host pushed for one run — a <c>SystemInput</c> or an
/// <c>ObserverInput</c> (same three vectors), looked up by declared param index.
/// </summary>
internal readonly struct ParamsView
{
    readonly SystemInput _sys;
    readonly ObserverInput _obs;
    readonly bool _isObserver;

    internal ParamsView(SystemInput sys)
    {
        _sys = sys;
        _obs = default;
        _isObserver = false;
    }

    internal ParamsView(ObserverInput obs)
    {
        _sys = default;
        _obs = obs;
        _isObserver = true;
    }

    int QueriesLength => _isObserver ? _obs.QueriesLength : _sys.QueriesLength;
    QueryRows? Queries(int i) => _isObserver ? _obs.Queries(i) : _sys.Queries(i);
    int ResourcesLength => _isObserver ? _obs.ResourcesLength : _sys.ResourcesLength;
    ResValue? Resources(int i) => _isObserver ? _obs.Resources(i) : _sys.Resources(i);
    int EventsLength => _isObserver ? _obs.EventsLength : _sys.EventsLength;
    EventValues? Events(int i) => _isObserver ? _obs.Events(i) : _sys.Events(i);

    /// <summary>The rows of the Query param at <paramref name="param"/> (empty when the host sent none).</summary>
    public QueryRowsView Rows(int param)
    {
        for (var i = 0; i < QueriesLength; i++)
            if (Queries(i) is { } q && q.ParamIndex == (uint)param)
                return new QueryRowsView(q);
        return default;
    }

    /// <summary>The Res / ResMut param's value; null when the host has no such resource now.</summary>
    public CompView? Resource(int param)
    {
        for (var i = 0; i < ResourcesLength; i++)
            if (Resources(i) is { } r && r.ParamIndex == (uint)param)
                return r.Value is { } v ? new CompView(v) : null;
        return null;
    }

    /// <summary>The Events param's values since the last run (count 0 when none).</summary>
    public EventValuesView EventValues(int param)
    {
        for (var i = 0; i < EventsLength; i++)
            if (Events(i) is { } e && e.ParamIndex == (uint)param)
                return new EventValuesView(e);
        return default;
    }
}

/// <summary>One Events param's values, indexed (no iterator allocation).</summary>
internal readonly struct EventValuesView
{
    readonly EventValues _e;
    readonly bool _has;

    internal EventValuesView(EventValues e)
    {
        _e = e;
        _has = true;
    }

    public int Count => _has ? _e.ValuesLength : 0;

    public CompView this[int i] => new(_e.Values(i)!.Value);
}

/// <summary>The rows of one query param.</summary>
internal readonly struct QueryRowsView
{
    readonly QueryRows _q;
    readonly bool _has;

    internal QueryRowsView(QueryRows q)
    {
        _q = q;
        _has = true;
    }

    public int Count => _has ? _q.RowsLength : 0;

    public RowView Row(int i) => new(_q.Rows(i)!.Value);
}

/// <summary>One query row: an entity plus its reading terms' components, in declaration order.</summary>
internal readonly struct RowView
{
    readonly Row _row;

    internal RowView(Row row) => _row = row;

    public ulong Entity => _row.Entity;

    public CompView? Comp(int i) =>
        (uint)i < (uint)_row.CompsLength ? new CompView(_row.Comps(i)!.Value) : null;
}

/// <summary>A component / resource / event payload.</summary>
internal readonly struct CompView
{
    readonly CompValue _c;

    internal CompView(CompValue c) => _c = c;

    public ReadOnlySpan<byte> Bytes => _c.GetDataBytes() is { } d ? d.AsSpan() : default;

    /// <summary>The JSON payload as <typeparamref name="T"/>; an empty payload (a marker) reads as <c>{}</c>.</summary>
    public T? Parse<T>(JsonTypeInfo<T> typeInfo)
    {
        var b = Bytes;
        return JsonSerializer.Deserialize(b.Length == 0 ? "{}"u8 : b, typeInfo);
    }
}
