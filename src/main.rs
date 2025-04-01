use json::{json_dynamic_dispatch, json_static_dispatch};

mod json;

fn main() {
    let json_str = r#"
    {
      "string": "Hello, World!",
      "integer": 123,
      "negative_integer": -456,
      "float": 3.14159,
      "negative_float": -2.718,
      "boolean_true": true,
      "boolean_false": false,
      "null_value": null,
      "empty_string": "",
      "special_characters": "!@#$%^&*()_+-=[]{}|;:'\",.<>?/`~",
      "unicode_string": "你好，世界！🌍",
      "base64_binary": "SGVsbG8gV29ybGQh",
      "date_iso": "2024-03-30T12:34:56Z",
      "date_epoch": 1711790096,
      "array": [1, "two", 3.0, false, null, {"nested_key": "value"}],
      "nested_object": {
        "level1": {
          "level2": {
            "level3": {
              "name": "Deep Nested",
              "value": 42
            }
          },
          "another_key": [true, false, "mixed"]
        }
      },
      "large_array": [
        {
          "id": 1,
          "name": "Item 1",
          "tags": ["red", "blue", "green"]
        },
        {
          "id": 2,
          "name": "Item 2",
          "tags": ["yellow", "purple"]
        },
        {
          "id": 3,
          "name": "Item 3",
          "tags": []
        }
      ],
      "empty_object": {},
      "empty_array": [],
      "deeply_nested_array": [[[["deep"]]]]
    }
    "#;

    assert_eq!(
        json_static_dispatch::parse(json_str),
        json_dynamic_dispatch::parse(json_str)
    );
}
