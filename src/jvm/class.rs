use enum_primitive::FromPrimitive;
use jvm::constant::Constant;
use jvm::constantpool::ConstantPool;
use jvm::field::Fields;
use jvm::field::Field;
use jvm::attribute::Attribute;
use jvm::attribute::Attributes;
use jvm::method::Methods;
use jvm::method::Method;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::iter;
use std::fmt;

#[derive(Clone,Default)]
pub struct Class {
	bytes: Vec<u8>,
	magic: u32,
	minor_version: u16,
	major_version: u16,
	constant_pool_count: u16,
	constant_pool: ConstantPool,
	access_flags: u16,
	this_class: u16,
	super_class: u16,
	interfaces_count: u16,
	interfaces: Vec<u16>,
	fields_count: u16,
	fields: Fields,
	methods_count: u16,
	methods: Methods,
	attributes_count: u16,
	attributes: Attributes,
}

impl Class {

	pub fn get_constant_pool(&self) -> &ConstantPool {
		&self.constant_pool
	}

	pub fn get_method(&self, method_name: &String) -> Option<&Method> {
		self.methods.get_by_name(&method_name, &self.constant_pool)
	}

	pub fn get_methods(&self) -> &Methods {
		&self.methods
	}

	pub fn get_name(&self) -> Option<String> {
		match self.constant_pool.get(self.this_class as usize) {
			Constant::Class(_, name_idx) => {
				match self.constant_pool.get(name_idx as usize) {
					Constant::Utf8(_, _, _, name) => {
							return Some(name);
						},
					_ => return None
				}
			},
			_ => return None
		}
	}

	fn load_constant_pool(c: &mut Class, offset: usize) -> usize {
		c.constant_pool = ConstantPool::from(&c.bytes[offset..].to_vec());
		c.constant_pool_count = c.constant_pool.constant_pool_count();
		offset + c.constant_pool.byte_len()
	}

	fn load_attributes(c: &mut Class, offset: usize)->usize {
		c.attributes = Attributes::from(&c.bytes[offset..].to_vec());
		c.attributes_count = c.attributes.attributes_count();
		offset + c.attributes.byte_len()
	}

	fn load_fields(c: &mut Class, offset: usize)->usize {
		c.fields = Fields::from(&c.bytes[offset..].to_vec());
		c.fields_count = c.fields.fields_count();
		offset + c.fields.byte_len()
	}

	fn load_methods(c: &mut Class, offset: usize) -> usize {
		c.methods = Methods::from(&c.bytes[offset..].to_vec());
		c.methods_count = c.methods.methods_count();
		offset + c.methods.byte_len()
	}

	pub fn load(class_with_path: &str) -> Option<Class> {
		let mut bytes: Vec<u8> = Vec::new();
		let mut c = Class::default();
		let mut offset : usize = 0;
		let mut fd: fs::File;

		match fs::File::open(class_with_path) {
			Ok(mut fd) => {
				if let Err(err) = fd.read_to_end(&mut bytes) {
					print!("oops: could not read the class file '{}': {}\n",
					       class_with_path,
					       err);
					return None;
				}
			},
			Err(err) => {
				print!("oops: could not read the class file '{}': {}\n",
				       class_with_path,
				       err);
				return None;
			}
		}

		c.bytes = bytes;

		c.magic = (c.bytes[offset + 0] as u32) << 24 |
		          (c.bytes[offset + 1] as u32) << 16 |
		          (c.bytes[offset + 2] as u32) << 8  |
		          (c.bytes[offset + 3] as u32) << 0;
		offset += 4;

		c.minor_version = (c.bytes[offset + 0] as u16) << 8 |
		                  (c.bytes[offset + 1] as u16) << 0;
		offset += 2;

		c.major_version = (c.bytes[offset + 0] as u16) << 8 |
		                  (c.bytes[offset + 1] as u16) << 0;
		offset += 2;

		/*
		 * Load the constants pool.
		 */
		println!("offset: {}\n", offset);
		offset = Class::load_constant_pool(&mut c, offset);
		println!("offset: {}\n", offset);

		c.access_flags = (c.bytes[offset+0] as u16) << 8 |
		                 (c.bytes[offset+1] as u16);
		offset+=2;

		c.this_class = (c.bytes[offset+0] as u16) << 8 |
		               (c.bytes[offset+1] as u16);
		offset+=2;

		c.super_class = (c.bytes[offset+0] as u16) << 8 |
		                (c.bytes[offset+1] as u16);
		offset+=2;

		c.interfaces_count = (c.bytes[offset+0] as u16) << 8 |
		                     (c.bytes[offset+1] as u16);
		offset+=2;

		print!("interfaces_count: {}\n", c.interfaces_count);
		/*
		 * Handle the interfaces.
		 */
		c.interfaces = iter::repeat(0 as u16).
		                    take(c.interfaces_count as usize).
		                    collect();
		for i in 1 .. c.interfaces_count as usize {	
			c.interfaces[i] = (c.bytes[offset+0] as u16) << 8 |
			                  (c.bytes[offset+1] as u16);
			offset+=2;
		}

		/*
		 * Now parse the fields.
		 */

		offset = Class::load_fields(&mut c, offset);

		/*
		 * Now parse the methods.
		 */
		offset = Class::load_methods(&mut c, offset);

		offset = Class::load_attributes(&mut c, offset);
		Some(c)
	}
}

impl fmt::Display for Class {
	fn fmt(&self, f: &mut fmt::Formatter) ->fmt::Result {
		write!(f,"size: {}\n", self.bytes.len());
		write!(f,"magic: {}\n", self.magic);
		write!(f,"minor_version: {}\n", self.minor_version);
		write!(f,"major_version: {}\n", self.major_version);
		write!(f,"constant_pool_count: {}\n", self.constant_pool_count);
		for i in 1 .. self.constant_pool_count {
			write!(f,"#{}: {}\n", i, self.constant_pool.get(i as usize));
		}
		write!(f,"access_flags: {}\n", self.access_flags);
		write!(f,"this_class: {}\n", self.this_class);
		write!(f,"super_class: {}\n", self.super_class);
		write!(f,"interfaces_count: {}\n", self.interfaces_count);
		for i in 1 .. self.interfaces_count  {
			write!(f,"#{}: {}\n", i, self.interfaces[i as usize - 1]);
		}
		write!(f,"fields_count: {}\n", self.fields_count);
		write!(f,"fields: {}\n", self.fields);
		write!(f,"methods_count: {}\n", self.methods_count);
		write!(f,"methods: {}\n", self.methods);
		write!(f,"attributes_count: {}\n", self.attributes_count);
		write!(f,"attributes: {}\n", self.attributes)
	}
}
