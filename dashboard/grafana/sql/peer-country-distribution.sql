WITH peer_ip_country AS (
    SELECT DISTINCT ON (network,
        bucket,
        ip)
        time_bucket (INTERVAL '1 hour', time) AS bucket,
        network,
        ip,
        country
    FROM
        peer
)
SELECT
    p.bucket AS time,
    p.country AS metric,
    count(*) AS value
FROM
    peer_ip_country AS p
WHERE
    p.network = '$network'
    AND $__timeFilter(p.bucket)
GROUP BY
    p.bucket,
    p.country
ORDER BY
    p.bucket;

