mod crypto;
mod fs;
mod http;
mod runtime;

use crypto::Crypto;
use fs::Fs;
use http::Http;
use runtime::{current, print, set_timeout, Runtime};

fn javascript() {
    print("First call to read test.txt");
    Fs::read("test.txt", |result| {
        let text = result.into_string().unwrap();
        let len = text.len();
        print(format!("First count: {} characters.", len));

        print(r#"I want to create a "magic" number based on the text."#);
        Crypto::encrypt(text.len(), |result| {
            let n = result.into_int().unwrap();
            print(format!(r#""Encrypted" number is: {}"#, n));
        })
    });

    print("Registering immediate timeout 1");
    set_timeout(0, |_res| {
        print("Immediate1 timed out");
    });
    print("Registering immediate timeout 2");
    set_timeout(0, |_res| {
        print("Immediate2 timed out");
    });

    print("Second call to read test.txt");
    Fs::read("test.txt", |result| {
        let text = result.into_string().unwrap();
        let len = text.len();
        print(format!("Second count: {} characters.", len));

        print("Third call to read test.txt");
        Fs::read("test.txt", |result| {
            let text = result.into_string().unwrap();
            print_content(&text, "file read");
        });
    });

    print("Registering a 3000 and a 500 ms timeout");
    set_timeout(3000, |_res| {
        print("3000ms timer timed out");
        set_timeout(500, |_res| {
            print("500ms timer(nested) timed out");
        });
    });

    print("Registering a 1000 ms timeout");
    set_timeout(1000, |_res| {
        print("SETTIMEOUT");
    });

    print("Registering http get request to google.com");
    Http::http_get_slow("http//www.google.com", 2000, |result| {
        let result = result.into_string().unwrap();
        print_content(result.trim(), "web call");
    });
}

fn main() {
    let rt = Runtime::new();
    rt.run(javascript);
}

fn print_content(t: impl std::fmt::Display, descr: &str) {
    println!(
        "\n===== THREAD {} START CONTENT - {} =====",
        current(),
        descr.to_uppercase()
    );

    let content = format!("{}", t);
    let lines = content.lines().take(2);
    let main_cont: String = lines.map(|l| format!("{}\n", l)).collect();
    let opt_location = content.find("Location");

    let opt_location = opt_location.map(|loc| {
        content[loc..]
            .lines()
            .next()
            .map(|l| format!("{}\n", l))
            .unwrap_or_default()
    });

    println!(
        "{}{}... [Note: Abbreviated for display] ...",
        main_cont,
        opt_location.unwrap_or_default()
    );

    println!("===== END CONTENT =====\n");
}
