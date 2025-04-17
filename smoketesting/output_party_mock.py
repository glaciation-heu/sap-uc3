from flask import Flask, Response, request

class EndpointAction(object):

    def __init__(self, action):
        self.action = action

    def __call__(self, *args):
        # Perform the action
        json = request.get_json()
        answer = self.action(json["collaborationId"], json["secretId"])
        # Create the answer (bundle it in a correctly formatted HTTP answer)
        self.response = Response(answer, status=200, headers={})
        # Send it
        return self.response

class FlaskAppWrapper(object):
    app = None

    def __init__(self, name):
        self.app = Flask(name)
    def run(self, port, handler):
        real_handler = lambda x,y: handler.notify(x,y)
        self.add_endpoint(endpoint="/notify", endpoint_name="notify", handler=real_handler)
        self.app.run(host="0.0.0.0", debug=False, port=port)

    def stop(self):
            func = request.environ.get('werkzeug.server.shutdown')
            if func is None:
                raise RuntimeError('Not running with the Werkzeug Server')
            func()
            print("Server shutting down!")

    def add_endpoint(self, endpoint=None, endpoint_name=None, handler=None):
        self.app.add_url_rule(endpoint, endpoint_name, EndpointAction(handler), methods=["PUT"])
        # You can also add options here : "... , methods=['POST'], ... "

def action(collabId, secretId):
    print(f"Notification received {collabId} {secretId}")

if __name__ == "__main__":
    a = FlaskAppWrapper("test_notify")
    a.add_endpoint(endpoint="/notify", endpoint_name="notify", handler=action)
    a.run()
