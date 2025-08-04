#!/bin/bash

cd MP-SPDZ

bit_precision_field=64
bit_precision_ring=64
bit_precision_yao=48 #10 records already suffice to generate ~36 bit numbers (must be larger for larger data sizes!)

prog_path="`pwd`/Programs/Source"

for prog in $prog_path/*; do
  prog=`basename $prog`
  prog=${prog%.mpc} #remove .mpc suffix

  if [[ "$prog" == *_yao ]]; then # program name ends with _yao (GC-specific program version)
    # yao
    ./compile.py -G -B $bit_precision_yao "$prog" #yao version (<prog>_yao)
  else
    # mod p
    ./compile.py -F $bit_precision_field $prog #field version (<prog>)
    # mod 2^k
    ln -s "$prog_path/${prog}.mpc" "$prog_path/${prog}_ring.mpc" # field program also works for ring (<prog>_ring)
    ./compile.py -R $bit_precision_ring "${prog}_ring"
  fi
done