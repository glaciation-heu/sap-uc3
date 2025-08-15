# SmokeTesting

A simple smoketesting skript which triggers a computation.

The computation is defined in `mpc_program.mpc` and the CarbyneStack configuration in `csconfig`. 
This smoke test works with both a mocked and real computation service (e.g. CarbyneStack)

### Environment Variables
| Variabel                    | Description                                                                        |
|-----------------------------|------------------------------------------------------------------------------------|
| `COORD_SERVICE_URI`         | URI of the client service                                                          |
| `CLIENT_SERVICE_URI`        | URI of the coordination service                                                    |
| `SMOKETESTING_INSTANCE_URI` | URI of the smoketesting instance. Is used to check if a notification was received. |
