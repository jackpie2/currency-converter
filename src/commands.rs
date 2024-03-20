use crate::{
    cache::Cache,
    helpers::error_and_exit,
    sources::{currency_api::CurrencyApi, ConverterDataSource},
    ConvertArgs,
};
use spinoff::{spinners, Color, Spinner};

pub async fn convert(args: &ConvertArgs) -> ! {
    let mut spinner = Spinner::new(spinners::Dots, "Loading the cache...", Color::White);
    let mut cache = match Cache::load() {
        Ok(val) => val,
        Err(_) => Cache::new(),
    };

    let base = args.base.to_uppercase();
    let target = args.target.to_uppercase();

    spinner.update(spinners::Dots, "Getting the data...", Color::White);
    let converter = match cache.get(&base, &target, args.cache_duration) {
        Some(val) => {
            spinner.stop_and_persist(
                ">",
                format!(
                    "Using cached data ({}s before considered stale).",
                    args.cache_duration
                )
                .as_str(),
            );
            val
        }
        None => {
            spinner.update(
                spinners::Dots,
                "Fetching data from the API...",
                Color::White,
            );
            let converter = match CurrencyApi::load(&base, &target).await {
                Ok(val) => val,
                Err(err) => {
                    spinner.stop_and_persist(">", "Failed to fetch data from the API.");
                    error_and_exit(err)
                }
            };
            cache.set(&converter);
            spinner.stop_and_persist(">", "Data successfully fetched from the API.");
            converter
        }
    };

    let result = converter.convert(args.amount);

    println!(
        "{amount} {base} = {result:.precision$} {target} (1 {base} ~= {rate:.precision$} {target})",
        amount = args.amount,
        base = base,
        result = result,
        target = target,
        precision = args.precision,
        rate = converter.rate
    );

    match cache.save() {
        Ok(_) => std::process::exit(0),
        Err(err) => error_and_exit(&err),
    }
}

pub async fn list() -> ! {
    let list = match CurrencyApi::list().await {
        Ok(val) => val,
        Err(err) => error_and_exit(err),
    };

    println!("{}", list);

    std::process::exit(0);
}

pub async fn interactive() -> ! {
    println!("Cache duration in seconds (300s): ");
    let mut cache_duration = String::new();
    std::io::stdin()
        .read_line(&mut cache_duration)
        .expect("Failed to read line");
    let cache_duration: u64 = cache_duration.trim().parse().unwrap_or(300);

    println!("Precision (2): ");
    let mut precision = String::new();
    std::io::stdin()
        .read_line(&mut precision)
        .expect("Failed to read line");

    let precision: usize = precision.trim().parse().unwrap_or(2);
    let mut cache = match Cache::load() {
        Ok(val) => val,
        Err(_) => Cache::new(),
    };

    loop {
        println!("Base currency: ");
        let mut base = String::new();
        std::io::stdin()
            .read_line(&mut base)
            .expect("Failed to read line");
        base = base.trim().to_string().to_uppercase();

        println!("Target currency: ");
        let mut target = String::new();
        std::io::stdin()
            .read_line(&mut target)
            .expect("Failed to read line");

        target = target.trim().to_string().to_uppercase();

        println!("Amount: ");
        let mut amount = String::new();
        std::io::stdin()
            .read_line(&mut amount)
            .expect("Failed to read line");

        let amount: f64 = match amount.trim().parse() {
            Ok(val) => val,
            Err(_) => {
                println!("Invalid amount, please try again.");
                continue;
            }
        };

        let mut spinner = Spinner::new(spinners::Dots, "Getting the data...", Color::White);
        let converter = match cache.get(&base, &target, cache_duration) {
            Some(val) => {
                spinner.stop_and_persist(
                    "\n>",
                    format!(
                        "Using cached data ({}s before considered stale).",
                        cache_duration
                    )
                    .as_str(),
                );
                val
            }
            None => {
                spinner.update(
                    spinners::Dots,
                    "Fetching data from the API...",
                    Color::White,
                );
                let converter = match CurrencyApi::load(&base, &target).await {
                    Ok(val) => val,
                    Err(err) => {
                        spinner.stop_and_persist("\n>", "Failed to fetch data from the API.");
                        println!("There was an error while fetching data from the API ({}). Please try again.", err);
                        continue;
                    }
                };
                cache.set(&converter);
                spinner.stop_and_persist("\n>", "Data successfully fetched from the API.");
                converter
            }
        };

        let result = converter.convert(amount);

        println!(
            "{amount} {base} = {result:.precision$} {target} (1 {base} ~= {rate:.precision$} {target})\n",
            amount = amount,
            base = base.trim(),
            result = result,
            target = target.trim(),
            precision = precision,
            rate = converter.rate
        );

        match cache.save() {
            Ok(_) => (),
            Err(err) => error_and_exit(&err),
        }
    }
}
