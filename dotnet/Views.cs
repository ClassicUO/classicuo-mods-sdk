using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using ModAbi;

namespace CuoModSdk;

/// <summary>Reader over a <c>mod_run</c> input buffer (PUSH model).</summary>
internal readonly struct SystemInputView
{
    readonly SystemInput _input;

    internal SystemInputView(SystemInput input) => _input = input;

    /// <summary>Host frame tick (for guest-side timers).</summary>
    public ulong Tick => _input?.Tick ?? 0;

    /// <summary>The rows of the <paramref name="n"/>th declared query (0-based, in declaration
    /// order). Null for an observer context, which carries no rows.</summary>
    public QueryRowsView? Query(int n) =>
        _input is { Queries: { } queries } && (uint)n < (uint)queries.Count
            ? new QueryRowsView(queries[n])
            : null;
}

/// <summary>The rows of one query param.</summary>
internal readonly struct QueryRowsView
{
    readonly QueryRows _q;

    internal QueryRowsView(QueryRows q) => _q = q;

    public int Count => _q.Rows?.Count ?? 0;

    public RowView? Row(int i) =>
        _q.Rows is { } rows && (uint)i < (uint)rows.Count ? new RowView(rows[i]) : null;

    public IEnumerable<RowView> Rows
    {
        get
        {
            if (_q.Rows is { } rows)
                foreach (var r in rows)
                    yield return new RowView(r);
        }
    }
}

/// <summary>One query result row: an entity plus its Ref/Mut components in declaration order.</summary>
internal readonly struct RowView
{
    readonly Row _row;

    internal RowView(Row row) => _row = row;

    public ulong Entity => _row.Entity;

    /// <summary>Component at declared Ref/Mut index <paramref name="i"/>.</summary>
    public CompView? Comp(int i) =>
        _row.Comps is { } comps && (uint)i < (uint)comps.Count ? new CompView(comps[i]) : null;
}

/// <summary>A single component slot on a <see cref="RowView"/>.</summary>
internal readonly struct CompView
{
    readonly CompValue _c;

    internal CompView(CompValue c) => _c = c;

    /// <summary>Raw payload bytes (typed sub-buffer or JSON utf8).</summary>
    public ReadOnlyMemory<byte> Bytes => _c.Data is { } d ? d : ReadOnlyMemory<byte>.Empty;

    /// <summary>
    /// Deserialize the JSON payload into <typeparamref name="T"/>; <c>default</c> on empty
    /// data. JsonTypeInfo overload only: source-gen metadata keeps this AOT-safe under ILC.
    /// </summary>
    public T? Parse<T>(JsonTypeInfo<T> typeInfo)
    {
        var b = Bytes;
        return b.Length == 0 ? default : JsonSerializer.Deserialize(b.Span, typeInfo);
    }
}

/// <summary>Reader over a <c>mod_observer</c> input buffer.</summary>
internal readonly struct ObserverInputView
{
    readonly ObserverInput _input;
    readonly ulong _entity;

    internal ObserverInputView(ObserverInput input, ulong entity)
    {
        _input = input;
        _entity = entity;
    }

    /// <summary>The triggering entity (authoritative — from the export arg).</summary>
    public ulong Entity => _entity;

    /// <summary>The event/component payload (Insert/Remove component, or Custom event JSON).</summary>
    // Null-conditional on _input: a system's params are built from a `default` observer
    // view, so this is reachable outside an observer call.
    public CompView? Value => _input?.Value is { } v ? new CompView(v) : null;
}
