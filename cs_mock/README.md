# CS Mock

This is a mock implementation for the [CarbyneStack API](https://carbynestack.io/documentation/reference/api-specification/).

Implemented are only the API calls used for the MPC computation e.g. managing secrets and starting MPC computations.

This implementation is used for enhanced testing of the [client service](../client_service) and [coordination_service](../coordination_service) without the need to deploy the CarbyneStack, which needs a lot of resources and multiple kubernetes environments.

## Standalone Run

Install the rust tool chain according to the documentation on [rust-lang.org](https://www.rust-lang.org/tools/install)

To start the service execute `cargo run`

Visit [http://localhost:80/docs](http://localhost:80/docs) for the interactive Swagger-UI documentation of the service.

### environment-variables

| variable | description | default |
| ---------|-------------|---------|
| `SERVICE_PORT` | specify the port the service will listen on | `80` |
| `SERVICE_ADDRESS` | Address of the service | `0.0.0.0` |
| `SWAGGER_SERVER_URI` | Addressed used in the swagger API for testing the API calls. | `http://$SERVICE_ADDRESS:$SERVICE_PORT` |

## Dockerization

To build the service as a docker image run

`docker build -t cs_mock:0.1.0 .`
