FROM python:3.14-rc-alpine

WORKDIR /usr/src/smoketesting
ADD . .
RUN pip install -r requirements.txt
ENV COORD_SERVICE_URI="http://coordination_service:80"
ENV CLIENT_SERVICE_URI="http://client_service:80"
ENV SMOKETESTING_INSTANCE_URI="http://smoketesting:80"

CMD ["python", "smoketesting.py"]