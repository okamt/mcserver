import json

def extract_list_names(json_string):
    try:
        data = json.loads(json_string)
        
        if not isinstance(data, dict):
            return "Error: Root of JSON must be an object"
        
        list_names = [key for key, value in data.items() if isinstance(value, list)]
        
        return list_names
    except json.JSONDecodeError:
        return "Error: Invalid JSON string"

with open("src/assets/registries.min.json", "r") as f:
  json_string = f.read()

result = extract_list_names(json_string)
print(result)

print("pub struct Registries<'a> {")
for name in result:
    changed_name = name.replace('/', '_')
    if name == changed_name:
        print(f"    #[serde(borrow)]\n    {name}: Vec<&'a str>,")
    else:
        print(f"    #[serde(rename=\"{name}\")]\n    #[serde(borrow)]\n    {changed_name}: Vec<&'a str>,")
print("}")