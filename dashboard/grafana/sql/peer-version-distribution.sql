WITH peer_ip_version AS (
    SELECT DISTINCT ON (network,
        bucket,
        ip)
        time_bucket (INTERVAL '1 hour', time) AS bucket,
        network,
        ip,
        version
    FROM
        peer
)
SELECT
    p.bucket AS time,
    p.version AS metric,
    count(*) AS value
FROM
    peer_ip_version AS p
WHERE
    p.network = '$network'
    AND $__timeFilter(p.bucket)
GROUP BY
    p.bucket,
    p.version
ORDER BY
    p.bucket;

