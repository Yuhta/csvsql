#!/bin/sh
set -e

TESTS_DIR=`dirname $0`
PROJ_DIR=$TESTS_DIR/..
SQLCSV=$PROJ_DIR/target/debug/sqlcsv
TMP_FILE=`mktemp`

$SQLCSV -t T:$TESTS_DIR/simple.csv 'select * from T where A = 2' >$TMP_FILE
printf 'A,B\n2,bar\n' | diff $TMP_FILE -

$SQLCSV -t T:$TESTS_DIR/simple.csv 'select sum (A) as S from T' >$TMP_FILE
printf 'S\n6\n' | diff $TMP_FILE -

$SQLCSV -t T1:$TESTS_DIR/simple.csv -t T2:$TESTS_DIR/simple2.csv \
    'select T1.A, B, C from T1 left join T2 on T2.A = T1.A' >$TMP_FILE
printf 'A,B,C\n1,foo,kkk\n2,bar,\n3,quux,jjj\n' | diff $TMP_FILE -
