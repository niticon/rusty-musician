use inline_python::python;

fn main() {
    let artist_name = String::new();
    python! {
    import lyricsgenius as lg

    token = input("Enter your Genius API client access token: ")
    api = lg.Genius(token)

    'artist_name = input("Enter an artist's name: ")
    artist = api.search_artist('artist_name, max_songs=1,
     include_features=False)
    artist.save_lyrics(extension="txt")
    }
}
