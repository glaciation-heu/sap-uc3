# Secure Collaborative Computation

The Secure Collaborative Computation consists of components for computation on encrypted data via secure multi-party computation (MPC).
For component details and MPC terminology refer to the [component descriptions](https://github.com/glaciation-heu/IceStream/tree/main/secure_collaborative_computation_service).


## Project overview

* [Charts](charts/) contains the HelmCharts used to deploy the secure collaborative computation service.

* [.github/workflows](.github/workflows/) contains definition of GitHub workflows.
    * Testing workflow is triggered when a new version `tag` is pushed. It will execute the unit tests of the [client_service](client_service), [computation_service](computation_service) and [cs_mock](cs_mock).
    * Docker deployment workflow is triggered if the testing workflow succeeds. This workflow will build the docker images used in the secure collaborative computation service and deploy them on the GitHub docker registry.
    * HelmChart deployment workflow ts triggered if the testing workflow succeeds. It will create a release for all HelmCharts used in the secure collaborative computation service and publish it to github-pages.

* [Client Service](client_service/README.md) implementation of the client service.

* [Coordination Service](coordination_service/README.md) implementation of the coordination service.

* [CS Mock](cs_mock/README.md) A mock implementation of [CarbyneStack](https://carbynestack.io/) used for testing. Due to the non-collusion assumption of MPC, the computation service is typically deployed in different computation environments (e.g., different cloud providers or regions). To simplify testing, a mock implementation is used to perform test operations.

## Local test deployment with docker compose

A `docker-compose.yaml` file is provided for an easy way to test out the services.

First install [docker](https://docs.docker.com/engine/install/) and the [docker compose plugin](https://docs.docker.com/compose/install/linux/).

Run

```bash
docker compose up -d
```

to build and run the services. After startup the services can be tested using the interactive OpenAPI documentations of the [Client Service](http://localhost:8081/docs), [Coordination Service](http://localhost:8082/docs) and the [CS Mock](http://localhost:8085/docs).

## Deployment with helm

Using the helm chart is straightforward.

First add the helm chart repository
`helm repo add scc-charts https://...`

Install the combined helm chart using
`helm install secure-collab-service secure-collab-service`

This will deploy a client service, the coordination service and a cs-mock instance for testing. If you would like to only deploy the client or coordination service use the `client_service` or `cordination_service` helm chart.
