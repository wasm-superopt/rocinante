#!/bin/bash

for i in {1..14} 
do
  time cargo run examples/hackers_delight/p$i.wat  --no-opti -t 300 enumerative  > /p$i.txt
done

