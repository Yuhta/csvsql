extern crate csv;
extern crate getopts;
extern crate rusqlite;

use csv::{Reader, Writer};
use getopts::Options;
use rusqlite::SqliteConnection;
use rusqlite::types::ToSql;
use std::io::{Read, Write, stdout, stderr};
use std::{env, process};

fn load_table<A: Read>(conn: &SqliteConnection,
                       table: &str,
                       mut reader: Reader<A>) {
    let hdrs = reader.headers().unwrap();
    let hdrs: Vec<_> = hdrs.into_iter().map(|h| format!("{} TEXT", h)).collect();
    let sql = format!("CREATE TABLE {} ({})", table, hdrs.connect(", "));
    if let Err(e) = conn.execute(&sql, &[]) {
        panic!("Error creating table `{}': {}", table, e);
    }
    let params = hdrs.iter().map(|_| "?").collect::<Vec<_>>();
    let sql = format!("INSERT INTO {} VALUES ({})", table, params.connect(", "));
    let mut stmt = conn.prepare(&sql).unwrap();
    for row in reader.records() {
        let row = row.unwrap();
        let row: Vec<_> = row.iter().map(|c| c as &ToSql).collect();
        stmt.execute(&row).unwrap();
    }
}

fn query<A: Write>(conn: &SqliteConnection,
                   sql: &str,
                   writer: &mut Writer<A>) {
    let mut stmt = match conn.prepare(sql) {
        Ok(s)  => s,
        Err(e) => panic!("Error preparing SQL: {}", e),
    };
    let hdrs: Vec<_> =
        stmt.column_names().into_iter().map(|s| s.to_string()).collect();
    let n = hdrs.len();
    writer.encode(hdrs).unwrap();
    let rows = stmt.query_map(&[], |row| {
        (0..n).map(|i| row.get(i as i32)).collect::<Vec<String>>()
    }).unwrap();
    for row in rows {
        let row = row.unwrap();
        writer.encode(row).unwrap();
    }
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let mut opts = Options::new();
    opts.optmulti("t", "table", "specify a table from a file", "TABLE:FILE");
    opts.optflag("h", "help", "display this help and exit");
    let args: Vec<_> = env::args().collect();
    let usage = format!("{} SQL", opts.short_usage(&args[0]));
    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(f) => {
            writeln!(stderr(), "{}\n{}", f, usage).unwrap();
            process::exit(1);
        }
    };
    if matches.opt_present("h") {
        println!("{}", opts.usage(&usage));
        return;
    }
    let tables = matches.opt_strs("t");
    let sql = match matches.free.len() {
        1 => &matches.free[0],
        _ => {
            writeln!(stderr(), "{}", usage).unwrap();
            process::exit(1);
        }
    };
    let conn = SqliteConnection::open_in_memory().unwrap();
    for t in tables {
        let tf: Vec<_> = t.splitn(2, ':').collect();
        if tf.len() != 2 {
            panic!("Invalid table definition: {}", t);
        }
        let reader = match Reader::from_file(tf[1]) {
            Ok(r)  => r,
            Err(e) => panic!("Error reading file `{}': {}", tf[1], e),
        };
        load_table(&conn, tf[0], reader);
    }
    let mut wtr = Writer::from_writer(stdout());
    query(&conn, sql, &mut wtr);
}

#[cfg(test)]
mod tests {
    use csv::{Reader, Writer};
    use rusqlite::SqliteConnection;

    #[test]
    fn test_load_table() {
        let conn = SqliteConnection::open_in_memory().unwrap();
        let rdr = Reader::from_string("A,B\n1,2\n3,4");
        super::load_table(&conn, "T", rdr);
        let mut stmt = conn.prepare("SELECT * FROM T").unwrap();
        let mut xss = stmt.query_map(&[], |row| (row.get(0), row.get(1))).unwrap();
        assert_eq!(xss.next().unwrap().unwrap(), (1, 2));
        assert_eq!(xss.next().unwrap().unwrap(), (3, 4));
        assert!(xss.next().is_none());
    }

    #[test]
    fn test_query() {
        let conn = SqliteConnection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE T (a INTEGER, b TEXT);
                            INSERT INTO T VALUES (1, 'foo');
                            INSERT INTO T VALUES (2, 'bar');").unwrap();
        let mut wtr = Writer::from_memory();
        super::query(&conn, "SELECT * FROM T", &mut wtr);
        assert_eq!(wtr.as_string(), "a,b\n1,foo\n2,bar\n");
    }
}
