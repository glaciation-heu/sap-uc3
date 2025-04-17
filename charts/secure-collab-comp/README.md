# Helm Chart for the secure collaborative computation service.

As of right now the images of the computation and coordination service are not deployed
on a publicly accassible repository.

## Local deployment for testing

Create a kubernetes cluster using [kind](https://kind.sigs.k8s.io/) by running

```bash
kind create cluster --name helmtest --image kindest/node:v1.26.6
```

Than build the images for the coordination and client service.

```bash
docker build -t coordination-service:0.1.0 coordination_service/
docker build -t client-service:0.1.0 client_service/
```

Now load the docker images to the registry of the `helmtest` kind environment.

```bash
kind load docker-image coordination-service:0.1.0 client-service:0.1.0 --name helmtest
```

Lastly deploy the environment by executing

```bash
helm install collab-computation-service .
```
