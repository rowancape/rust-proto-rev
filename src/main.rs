use std::fs;


#[derive(Debug)]
enum FieldData {
  VarInt { std_encoded_int: i64, zigzag_encoded_int: i64 },
  I64 { std_encoded_int: i64, double: f64 },
  I32 { std_encoded_int: i32, float: f32 },
  LengthDelimitedString(String),
  ImbeddedMessage(Box<Field>),
  LenDelimitedBytes(Vec<u8>),
}


#[derive(Debug)]
struct Field {
  field_key: u64,
  wire_type: u8,
  data: FieldData,
}


fn get_field_tag_data(data: &Vec<u8>, working_index: &mut usize) -> (u8, u64) {
  let mut field_key_le_bytes: Vec<u8> = Vec::new();
  let mut field_key: u64 = 0;
  
  let first_byte = data[*working_index];
  let wire_type = first_byte & 0b0000_0111;
  

  // If the continuation bit on the first byte is 0 then we return here
  if (first_byte >> 7) == 0 {
    field_key = (first_byte >> 3) as u64;
    return (wire_type, field_key)
  }
  
  // field_key_le_bytes.push((first_byte & 0b0111_1111) >> 3);

  loop {
    // Increment working index and read next byte of field key
    *working_index += 1;
    let byte = data[*working_index];
    field_key_le_bytes.push(byte & 0b0111_1111);

    // If the continuation bit is 0 break out of loop
    if (byte >> 7) == 0 {
      break;
    }
  }

  // Convert to big endian, remove MSBs, concatonate and interpret as u64
  for &byte in field_key_le_bytes.iter().rev() {
    field_key = (field_key << 7) | (byte & 0b0111_1111) as u64;
  }
  // The four bits from first_byte need to be added directly to the end with no leading zeroes
  field_key = (field_key << 4) | ((first_byte >> 3) & 0b0000_1111) as u64;

  // Increment working index one more time to place it on the first byte of field data.
  // Then return the big field_key
  *working_index += 1;
  return (wire_type, field_key)
}


fn read_varint(data: &Vec<u8>, working_index: &mut usize) -> (i64, i64){
  let mut varint_le_bytes: Vec<u8> = Vec::new();
  // let mut possible_int_values: Vec<i64> = Vec::new();
  
  loop {
    let byte = data[*working_index];
    varint_le_bytes.push(byte);

    *working_index += 1;

    if (byte >> 7) == 0 {
      break;
    }
  }

  let mut varint: u64 = 0;

  for &byte in varint_le_bytes.iter().rev() {
    varint = (varint << 7) | (byte & 0b0111_1111) as u64;
  }

  let std_encoded_int = varint as i64;
  let zigzag_encoded_int = ((varint >> 1) as i64) ^ -((varint & 1) as i64);

  (std_encoded_int, zigzag_encoded_int)
}


fn read_fixed_32(data: &Vec<u8>, working_index: &mut usize) {
  
}


fn main() {
  let data: Vec<u8> = fs::read("test.bin").expect("Failed to read file");

  let mut working_index: usize = 0;

  loop {
    if working_index + 1 >= data.len() {
      println!("End of file");
      break;
    }

    let (wire_type, field_key) = get_field_tag_data(&data, &mut working_index);

    if working_index == 2 {
      println!("{:<20}{:<20}{:<10}", "Field key", "Wire type", "Possible values");
    }

    match wire_type {
      0b0000_0000 => {
        let (std_encoded_int, zigzag_encoded_int) = read_varint(&data, &mut working_index);
        println!("{:<20}{:<20}{:<10}{:<20}", field_key, wire_type, std_encoded_int, zigzag_encoded_int);
      }

      _ => {
        println!("Encountered invalid wire type!");
        break;
      }
    }
  }
}