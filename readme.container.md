# Use realm in container

Github workflow pushes the images to [Github Container Registry](https://ghcr.io).

Image list:

- `ghcr.io/zhboner/realm:latest` standard realm image
- `ghcr.io/zhboner/realm-slim:latest` tailored realm image with only TCP+UDP support

## Notice
The [package](https://github.com/zhboner/realm/pkgs/container/realm) is no longer accessible. See https://github.com/zhboner/realm/issues/61.

As it was mentioned in the [comment](https://github.com/zhboner/realm/issues/61#issuecomment-1145760482) that, users could build their own images based on the released binaries:
```dockerfile
FROM alpine:latest

ARG VERSION="v2.3.4"

WORKDIR /realm

RUN wget https://github.com/zhboner/realm/releases/download/${VERSION}/realm-x86_64-unknown-linux-musl.tar.gz \
  && tar -zxvf realm-x86_64-unknown-linux-musl.tar.gz \
  && cp realm /usr/bin/realm \
  && chmod +x /usr/bin/realm

ENTRYPOINT ["/usr/bin/realm"]
```

**If there are non-official images, be careful and use at your own risk.**

## Docker

```bash
docker run -d -p 9000:9000 ghcr.io/zhboner/realm:latest -l 0.0.0.0:9000 -r 192.168.233.2:9000
```

## Docker Swarm (Docker Compose)

```yaml
# ./realm.yml
version: '3'
services:
  port-9000:
    image: ghcr.io/zhboner/realm:latest
    ports:
      - 9000:9000
    command: -l 0.0.0.0:9000 -r 192.168.233.2:9000
```

```bash
docker-compose -f ./realm.yml -p realm up -d
```

## Kubernetes

```yaml
# ./realm.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: realm-demo-deployment
  labels:
    app: realm
  namespace: default
spec:
  replicas: 1
  selector:
    matchLabels:
      app: realm 
  template:
    metadata:
      labels:
        app: realm 
    spec:
      containers:
      - name: realm
        image: ghcr.io/zhboner/realm:latest
        args:
          - "-l=0.0.0.0:9000"
          - "-r=192.168.233.2:9000"
        ports:
        - containerPort: 9000
        resources:
          requests:
            memory: "64Mi"
            cpu: "250m"
          limits:
            memory: "128Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: realm-lb
  namespace: default
spec:
  type: LoadBalancer
  selector:
    app: realm
  ports:
    - name: edge
      port: 9000
```
