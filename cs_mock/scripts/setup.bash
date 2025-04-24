#!/bin/bash

# install mpspdz
MP_SPDZ_VERSION="0.3.6"
curl -L https://github.com/data61/MP-SPDZ/releases/download/v$MP_SPDZ_VERSION/mp-spdz-$MP_SPDZ_VERSION.tar.xz | tar xJv
mv mp-spdz-$MP_SPDZ_VERSION MP-SPDZ
cd MP-SPDZ
rm Programs/Source/* #remove examples (simplifies compiling own programs)
Scripts/tldr.sh
Scripts/setup-ssl.sh 3 #for honest majority protocols