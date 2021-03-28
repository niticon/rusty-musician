import lyricsgenius as lg
import json

token = input('Enter your genius developer API token: ')
user_input = input('Select an artist: ')

genius = lg.Genius(token)

artist = genius.search_artist(user_input, max_songs=10, sort="title", include_features=False)

print(artist.songs)
all_lyrics = artist.save_lyrics()

json_object = json.loads(all_lyrics)
json_formatted_str = json.dumps(lyrics_json, indent=2)
print(json_formatted_str)


