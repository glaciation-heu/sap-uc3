#!/bin/python
###
## coordination-service smoke testing.
## This tests are intended to run after the secure-collab computation service was deployed to check if it runs correctly.
import os, time
import requests
import logging, coloredlogs
from requests_toolbelt.multipart.encoder import MultipartEncoder
from threading import Thread
from output_party_mock import EndpointAction, FlaskAppWrapper
import urllib.parse
from multiprocessing import Process, Manager
from urllib.parse import urlparse


logger = logging.getLogger(__name__)
coloredlogs.install()

logger.setLevel("INFO")

coord_service_uri = os.getenv("COORD_SERVICE_URI")
client_service_uri = os.getenv("CLIENT_SERVICE_URI")
testinginstance_uri = os.getenv("SMOKETESTING_INSTANCE_URI")
# regex to extract host and port information
testinstance_port = urlparse(testinginstance_uri).port
if testinstance_port == None:
    testinstance_port = 80
logger.info("Start smoketesting")


# Create collaboration
mp_encoder = MultipartEncoder(
fields={
    "name": "smoketesting",
    "number_of_parties": "1",
    "csv_header_line": "data",
    "mpc_program": ("mpc_program.mpc", open("mpc_program.mpc", 'rb'), 'text/plain'),
    "cs_config": ("cs_config", open("csconfig", 'rb'), 'text/plain')
}
)
resp = requests.post(f"{coord_service_uri}/collaboration", 
                     headers={"Content-Type": mp_encoder.content_type, "accept": "application/json"},
                     data=mp_encoder)
if resp.status_code != 200:
    logger.error(f"Error creating new collaboration. POST-Request on {coord_service_uri}/collaboration finished with status-code: {resp.status_code}\n", resp.text)
    exit(1)

collab_resp = resp.json()
collab_id = collab_resp["id"]
logger.info(f"Collaboration with id: {collab_id} was created")
def clean_create_collab():
    # remove coordination
    resp = requests.delete(f"{coord_service_uri}/collaboration/{collab_id}")
    if resp.status_code != 200:
        logger.error(f"Error while removing collaboration with id: {collab_id}\n", resp.text)
    else:
        logger.info(f"Collaboration with id: {collab_id} was successfully removed")


# Register party for collaboration
resp = requests.post(f"{coord_service_uri}/collaboration/{collab_id}/register-input-party/1", headers={"accept": "application/json"})
if resp.status_code != 200:
    logger.error(f"Error registering party for collaboration {collab_id}. Collaboration Service responded with status code: {resp.status_code}",resp.text)
    clean_create_collab()
    exit(1)

def clean_register_party():
    resp = requests.delete(f"{coord_service_uri}/collaboration/{collab_id}/register-input-party/1")
    if resp.status_code != 200:
        logger.error(f"Error while removing registered party from collaboration {collab_id}\t {resp.text}", )
    else:
        logger.info(f"Participation for collaboration with id: {collab_id} was successfully removed")
    clean_create_collab()
logger.info(f"Party 1 successfully registered to collaboration with id {collab_id}")

global collabId
collabId = None
global secretId 
secretId = None

class NotifcationHandler:
    def __init__(self, state):
        self.state = state
    def notify(self, collab_id, secret_id):
        self.state['collabId'] = collab_id
        self.state['secretId'] = secret_id

    def get(self, key):
        if key in self.state:
            return self.state[key]
        return None

with Manager() as manager:
    share_state = manager.dict()
    handler = NotifcationHandler(share_state)

    # Create dev output-party notification service
    outputPartyService = FlaskAppWrapper("debug")
    thread = Process(target=outputPartyService.run, args=(testinstance_port, handler, ))
    # thread = Thread(target=outputPartyService.run)
    thread.start()
    logger.info("Started testing thread, wait 2 seconds untill service is up")
    time.sleep(2)

    # Register as output party
    output_party_url = urllib.parse.quote_plus(testinginstance_uri)
    resp = requests.post(f"{coord_service_uri}/collaboration/{collab_id}/register-output-party/1?party_client_endpoint={output_party_url}", headers={"accept": "application/json", "Content-Type": "application/json"})
    if resp.status_code != 200:
        thread.terminate()
        logger.error(f"Error registering party for collaboration {collab_id}. Collaboration Service responded with status code: {resp.status_code}",resp.text)
        clean_create_collab()
        exit(1)

    # Upload test data
    mp_encoder = MultipartEncoder(
        fields={
            "data_csv": ("data_csv", open("testdata.csv", 'rb'), 'text/plain'),
        }
    )
    resp = requests.post(f"{client_service_uri}/secrets/{collab_id}/1", headers={"Content-Type": mp_encoder.content_type, "accept": "application/json"}, data=mp_encoder)
    if resp.status_code != 200:
        logger.error(f"Error uploading secrets to client service. {resp.text}")
        clean_register_party()
        exit(1)

    secret_id = resp.json()[0]
    logger.info(f"secret id: {secret_id}")
    def clean_upload_data():
        thread.terminate()
        clean_register_party()

    # Wait until notification was received
    timesSlept = 0
    while handler.get("collabId") == None and handler.get("secretId") == None:
        logger.info("Waiting for notification")
        time.sleep(2)
        if timesSlept > 5:
            logger.error(f"Notification not received after {2*timesSlept} seconds.")
            clean_upload_data()
            exit(1)
        timesSlept = timesSlept + 1
    thread.terminate()
    collabId = handler.get("collabId")
    secretId = handler.get("secretId")
    logger.info(f"Notification received from collaboration {collabId} wit result-secret {secretId}")

    # retrieve results
    logger.info("Check if result is available.")
    result_resp = requests.get(f"{client_service_uri}/result/{collab_id}/1", headers={"accept": "application/json"})

    if result_resp.status_code != 200:
        logger.error(f"Error retrieving results. Client-Service responded with status code {result_resp.status_code} {result_resp.text}")
        clean_upload_data()
        exit(1)

    logger.info("Smoketesting successfull!")
    logger.info(result_resp.json())
    clean_upload_data()
