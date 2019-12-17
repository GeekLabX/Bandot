
### build local ui

```
yarn
yarn start
```

You can access the UI via http://localhost:3000

### Building the bandot ui image

To build your own image from the source, you can run the following command:
```bash
../docker/build-ui.sh
```

### start docker container

```
cd ..
docker-compose -f docker/docker-compose.yml up -d
```

You can access the UI via http://localhost:3000
