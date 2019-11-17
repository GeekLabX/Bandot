# 1. Use docker to download latest images

```
 docker-compose -f docker/docker-compose-local.yml up -d
 
# Recreating docker_node_alice_1 ... done
# Recreating docker_node_bob_1   ... done
# Creating docker_bandot-ui_1    ... done

```
![](https://tva1.sinaimg.cn/large/006y8mN6gy1g90t7xs47ij31960i6qev.jpg)

# 2. open chrome
http://localhost:3000/#/explorer


# 3. bandot
## 3.1 init
init must run first, it will also set admin
![](https://tva1.sinaimg.cn/large/006y8mN6gy1g90uf2hobbj31v00aomze.jpg)

## 3.2 mint burn transfer
must be admin, mint first, burn and transfer to others
![](https://tva1.sinaimg.cn/large/006y8mN6gy1g91556cc4tj329y0kgq7k.jpg)

## 3.3 deposit and exchange
deposit into bdt, for stake
![](https://tva1.sinaimg.cn/large/006y8mN6gy1g90v0fl7gkj31wi0qkn1j.jpg)

## 3.4 setFee
it's 1k. fee rate is 10/1000
![](https://tva1.sinaimg.cn/large/006y8mN6gy1g90v1k5shvj31o70u0aev.jpg)

# 4. bancor
## 4.1 init bancor
input base token cw1k

![](https://tva1.sinaimg.cn/large/006y8mN6gy1g90urasrwgj31pq0l4acm.jpg)

## 4.2 buy and sell token
This is for bancor input and output. because I didn't find float number. It is saved for further improvement

![](https://tva1.sinaimg.cn/large/006y8mN6gy1g90uuiqdt6j31sk0u0tcg.jpg)

it changes base and token