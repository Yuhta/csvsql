SQLCSV
======

[![Build Status](https://travis-ci.org/Yuhta/csvsql.svg?branch=master)](https://travis-ci.org/Yuhta/csvsql)

SQLCSV is a command line tool to manipulate CSV file using SQL query.

## Example

```sh
$ cat a.csv
A,B
1,foo
2,bar

$ cat b.csv
A,C
1,kkk
2,jjj

$ csvsql -t T1:a.csv -t T2:b.csv 'select T1.A, B, C from T1 join T2 on T2.A = T1.A'
A,B,C
1,foo,kkk
2,bar,jjj
```
