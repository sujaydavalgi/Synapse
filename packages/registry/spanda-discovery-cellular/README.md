# spanda-discovery-cellular

Optional **LTE/5G modem** discovery transport for fleet agents reporting cellular connectivity.

## Status

**Experimental** — env override (`SPANDA_DISCOVERY_CELLULAR_MATCHES`) or host `mmcli -L` when ModemManager is installed.

## API

```bash
curl 'http://127.0.0.1:8080/v1/discovery?transport=cellular'
```

## Configuration

```bash
export SPANDA_DISCOVERY_CELLULAR_MATCHES="lte-modem-1@10.0.0.1"
```
