const SNIPPET: &str = include_str!("../snippet.txt");

fn main() {
    let mut args = std::env::args().skip(1);
    let vowel = args.next().expect("need vowel");
    let num: u32 = args.next().expect("need index").parse().unwrap();

    'lines: for line in SNIPPET.lines() {
        if line.contains("$VOWEL") {
            println!("{}", line.replace("$VOWEL", &vowel));
            continue 'lines;
        }

        for i in 0..24 {
            let name = i + 1;
            let name = format!("${name:02}");

            if line.contains(&name) {
                println!("{}", line.replace(&name, &format!("{}", num + i)));
                continue 'lines;
            }
        }
            
        println!("{line}");
    }
}
