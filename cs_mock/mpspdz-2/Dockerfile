FROM python:3.9.16-bullseye
WORKDIR /root
RUN apt-get update && apt-get install -y \
  curl \
  openssl \
  xz-utils \
  build-essential \
  clang \
  cmake  \
  automake \
  git \
  libboost-dev \
  libboost-filesystem-dev \
  libboost-iostreams-dev \
  libboost-thread-dev \
  libgmp-dev \
  libntl-dev \
  libsodium-dev \
  libssl-dev \
  libtool

ADD scripts .

# ADD api-srv api-srv
RUN ["bash", "setup.bash"]

RUN git clone https://github.com/DLTcollab/sse2neon.git /root/MP-SPDZ/deps/sse2neon
WORKDIR /root/MP-SPDZ/deps/sse2neon
RUN git checkout 29716df957401e9c357348e9b73c95a38cdcd34f
WORKDIR /root

EXPOSE 8011
# RUN rm -r ./MP-SPDZ/Programs/Source/*

ADD source/* ./MP-SPDZ/Programs/Source/
ADD client-interface.py ./MP-SPDZ/ExternalIO/client-interface.py 

WORKDIR /root/MP-SPDZ

# RUN make -j online

#RUN Scripts/setup-ssl.sh 2
#RUN Scripts/setup-clients.sh 3
RUN pip install gmpy2
# 
# RUN make bankers-bonus-client.x

ADD ./execute.sh .
CMD [ "./execute.sh" ]
# RUN ["bash", "compile.bash"]
# CMD [ "run_mpc.bash", "mascot", "secure-comparison", "2", "/tmp/status", "$SERVER_IP" ]
