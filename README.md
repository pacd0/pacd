# pacd
Proxy auto-config (PAC) server 


## Install

```shell script
cargo install pacd
```

## Usage

```shell script

# by default, use 'SOCKS5 127.0.0.1:1080' as proxy, listen at '127.0.0.1:8080'
pacd domains.txt

# set listen address
pacd -l 0.0.0.0:8080 domains.txt

# set proxy address
pacd -p "SOCKS5 127.0.0.1:9080" domains.txt

```
