use error_chain::*;

error_chain! {
    foreign_links {
        ParseFloat(std::num::ParseFloatError);
        ParseDate(chrono::format::ParseError);
        Io(std::io::Error);
        Reqwest(reqwest::Error);
        //ImageErr(image::ImageError);
    }

    errors { RandomResponseError(t: String) }
}
