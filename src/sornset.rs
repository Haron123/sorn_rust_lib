use std::{cell::RefCell, f64::{INFINITY, NEG_INFINITY}, rc::Rc};
use std::hash::{DefaultHasher, Hash, Hasher};
use fxhash::FxHashMap;

use crate::sorn::{Sorn, SornBitsType};

#[derive(Clone, Copy)]
pub enum SornValue
{
	Empty,
	Open((f64, f64)),
	OpenLeft((f64, f64)),
	OpenRight((f64, f64)),
	Exact(f64),
	PlusMinusInf,
}

impl SornValue
{
	/* Only works if the Value is exact, undefined for the rest */
	pub fn get(&self) -> Option<f64>
	{
		match self
		{
			SornValue::Empty =>
			{
				None
			}
			SornValue::Open(_) =>
			{
				None
			},

			SornValue::OpenLeft(_) =>
			{
				None
			},

			SornValue::OpenRight(_) =>
			{
				None
			},

			SornValue::Exact(value) =>
			{
				Some(*value)
			},

			SornValue::PlusMinusInf =>
			{
				None
			},
		}
	}

	pub fn min(&self) -> f64
	{
		match self
		{
			SornValue::Empty =>
			{
				0.0
			},

			SornValue::Open((start, _end)) =>
			{
				*start
			},

			SornValue::OpenLeft((start, _end)) =>
			{
				*start
			},

			SornValue::OpenRight((start, _end)) =>
			{
				*start
			},

			SornValue::Exact(value) =>
			{
				*value
			},

			SornValue::PlusMinusInf =>
			{
				NEG_INFINITY
			},
		}
	}

	pub fn max(&self) -> f64
	{
		match self
		{
			SornValue::Empty =>
			{
				0.0
			},

			SornValue::Open((_start, end)) =>
			{
				*end
			},

			SornValue::OpenLeft((_start, end)) =>
			{
				*end
			},

			SornValue::OpenRight((_start, end)) =>
			{
				*end
			},

			SornValue::Exact(value) =>
			{
				*value
			},

			SornValue::PlusMinusInf =>
			{
				INFINITY
			},
		}
	}

	pub fn is_exact(&self) -> bool
	{
		matches!(self, SornValue::Exact(_))
	}

	pub fn is_interval(&self) -> bool
	{
		self.is_open() ||
		self.is_leftopen() ||
		self.is_rightopen()
	}

	pub fn is_open(&self) -> bool
	{
		matches!(self, SornValue::Open(_))
	}

	pub fn is_leftopen(&self) -> bool
	{
		matches!(self, SornValue::OpenLeft(_))
	}

	pub fn is_rightopen(&self) -> bool
	{
		matches!(self, SornValue::OpenRight(_))
	}

	pub fn is_pminf(&self) -> bool
	{
		matches!(self, SornValue::PlusMinusInf)
	}
}

impl std::cmp::PartialEq for SornValue
{
	fn eq(&self, other: &Self) -> bool 
	{
		match (self, other) {
			(Self::Open(l0), Self::Open(r0)) => l0 == r0,
			(Self::OpenLeft(l0), Self::OpenLeft(r0)) => l0 == r0,
			(Self::OpenRight(l0), Self::OpenRight(r0)) => l0 == r0,
			(Self::Exact(l0), Self::Exact(r0)) => l0 == r0,
			_ => core::mem::discriminant(self) == core::mem::discriminant(other),
		}
	}
}

impl std::cmp::Eq for SornValue
{}

impl std::hash::Hash for SornValue
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) 
	{
		use SornValue::*;

		match self
		{
			Empty =>
			{
				0u8.hash(state);
			},
			Open((min, max)) =>
			{
				1u8.hash(state);
				min.to_bits().hash(state);
				max.to_bits().hash(state);
			},
			OpenLeft((min, max)) =>
			{
				2u8.hash(state);
				min.to_bits().hash(state);
				max.to_bits().hash(state);
			},

			OpenRight((min, max)) =>
			{
				3u8.hash(state);
				min.to_bits().hash(state);
				max.to_bits().hash(state);
			},
			Exact(val) =>
			{
				4u8.hash(state);
				val.to_bits().hash(state);
			},
			PlusMinusInf =>
			{
				5u8.hash(state);
			},
		}
	}
}

impl std::cmp::PartialOrd for SornValue
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> 
	{
        if self.max() < other.min() 
		{
            Some(std::cmp::Ordering::Less)
        }
		else if self.min() > other.max() 
		{
            Some(std::cmp::Ordering::Greater)
        }
		else if self.min() == other.min() && self.max() == other.max() 
		{
            Some(std::cmp::Ordering::Equal)
        }
		else 
		{
            None // Opens overlap, not totally orderable
		}
    }
}


impl std::fmt::Debug for SornValue
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self
		{
			SornValue::Empty =>
			{
				write!(f, "Empty SornValue")
			},

			SornValue::Open((start, end)) =>
			{
				write!(f, "({},{})", start, end)
			},

			SornValue::OpenLeft((start, end)) =>
			{
				write!(f, "({},{}]", start, end)
			},

			SornValue::OpenRight((start, end)) =>
			{
				write!(f, "[{},{})", start, end)
			},

			SornValue::Exact(value) =>
			{
				write!(f, "[{}]", value)
			},

			SornValue::PlusMinusInf =>
			{
				write!(f, "[±inf]")
			},
		}
	}
}

impl std::fmt::Display for SornValue
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
	{
		/* Use Debug Implementation */
		write!(f, "{:?}", self)
	}
}

const MAX_SETS: usize = 128;

#[derive(Clone)]
pub struct SornSet
{
	/* Key is own Bits, Value is result Bits */
	pub precomputed_pow: FxHashMap<SornBitsType, SornBitsType>,

	/* Key is (self.bits, operand.bits), Value is result Bits */
	pub precomputed_add: FxHashMap<(SornBitsType, SornBitsType), SornBitsType>,
	pub precomputed_sub: FxHashMap<(SornBitsType, SornBitsType), SornBitsType>,
	pub precomputed_mul: FxHashMap<(SornBitsType, SornBitsType), SornBitsType>,
	pub precomputed_div: FxHashMap<(SornBitsType, SornBitsType), SornBitsType>,

	pub sets: Vec<SornValue>,
	pub contains_inf: bool,
	pub one_bit: SornBitsType,
}

impl SornSet
{
	pub fn default() -> Self
	{
		SornSet
		{
			precomputed_pow: FxHashMap::default(),
			
			precomputed_add: FxHashMap::default(),
			precomputed_sub: FxHashMap::default(),
			precomputed_mul: FxHashMap::default(),
			precomputed_div: FxHashMap::default(),

			sets: Vec::with_capacity(MAX_SETS),
			contains_inf: false,
			one_bit: 0,
		}
	}

	pub fn new(start: f64, end: f64, step: f64, has_inf: bool) -> Self
	{
		/* Calculate number of ranges and bit size of sorn */
		let num_sets = ((end - start) / step).floor() as u32;

		/* Add Ranges to Vector */
		let mut sets = SornSet::default();

		if has_inf 
		{
			sets.contains_inf = true;
			//sets.push(SornValue::PlusMinusInf);
			sets.push(SornValue::Open((f64::NEG_INFINITY, start)));
		}

		for i in 0..num_sets
		{
			let first = (i as f64 * step) + start;
			let second = first + step;

			sets.push(SornValue::Exact(first));
			sets.push(SornValue::Open((first, second)));
		}
		sets.push(SornValue::Exact(end));

		if has_inf 
		{
			sets.push(SornValue::Open((end, f64::INFINITY)));
		}

		let one_bit = Sorn::sorn_to_bits(Rc::new(RefCell::new(sets.clone())), &SornValue::Exact(1.0));
		sets.one_bit = one_bit;

		sets
	}

	/* 
	[x] is Exact,  
	(x, x) is Open,
	[x,x) is LeftOpen,
	(x,x] is RightOpen,
	Values Seperated by Semicolon
	*/
	pub fn from_string(string: &str) -> Self
	{
		let mut sets = SornSet::default();
		let unums: Vec<&str> = string.split(";").collect();

		for unum in unums
		{
			let values: Vec<&str> = unum.split(",").collect();
			
			if values.len() == 1
			{
				let number: String = values[0].chars().filter(|&c| c != '[' && c != ']').collect();
				sets.push(SornValue::Exact(number.parse().unwrap()));
			}
			else if values.len() == 2
			{
				let mut left_open = false;
				let mut right_open = false;

				if values[0].contains('(')
				{
					left_open = true;
				}

				if values[1].contains(')')
				{
					right_open = true;
				}

				let mut first_value = values[0].chars();
				first_value.next();
				let first_value = first_value.as_str().parse().unwrap();

				let mut second_value = values[1].chars();
				second_value.next_back();
				let second_value = second_value.as_str().parse().unwrap();

				if left_open && right_open
				{
					sets.push(SornValue::Open((first_value, second_value)));
				}
				else if left_open
				{
					sets.push(SornValue::OpenLeft((first_value, second_value)));
				}
				else if right_open
				{
					sets.push(SornValue::OpenRight((first_value, second_value)));
				}
			}
		}

		let one_bit = Sorn::sorn_to_bits(Rc::new(RefCell::new(sets.clone())), &SornValue::Exact(1.0));
		sets.one_bit = one_bit;

		sets
	}

	pub fn len(&self) -> usize
	{
		self.sets.len()
	}

	pub fn first(&self) -> Option<&SornValue>
	{
		if self.len() > 0
		{
			Some(&self.sets[0])
		}
		else
		{
			None
		}
	}

	pub fn last(&self) -> Option<&SornValue>
	{
		if self.len() > 0
		{
			Some(&self.sets[self.len()-1])
		}
		else
		{
			None
		}
	}

	pub fn is_empty(&self) -> bool
	{
		self.len() == 0
	}

	pub fn push(&mut self, item: SornValue)
	{
		self.sets.push(item);
	}

	pub fn get(&self, index: usize) -> SornValue
	{
		self.sets[index].clone()
	}

	pub fn get_min_range(&self) -> Option<SornValue>
	{
		let ranges = &self.sets;

		if self.len() == 0
		{
			return None;
		}

		return Some(ranges[0]);
	}

	pub fn get_max_range(&self) -> Option<SornValue>
	{
		let ranges = &self.sets;

		if ranges.len() == 0
		{
			return None;
		}

		return Some(ranges[self.len()-1]);
	}

	pub fn get_sets_between(&self, range: SornValue) -> SornSet
	{
		let mut result = SornSet::default();

		for item in &self.sets
		{
			if (range.is_interval() && item.is_interval()) && (range.min() < item.max() && item.min() < range.max()) ||
			(range.is_interval() && item.is_exact()) && (range.min() <= item.get().unwrap() && range.max() >= item.get().unwrap()) ||
			(item.is_pminf() && range.is_pminf() && self.contains_inf)
			{
				result.push(item.clone());
			}
		}
		
		return result;
	}
}

impl std::cmp::PartialEq for SornSet
{
	fn eq(&self, other: &Self) -> bool 
	{
		self.sets == other.sets && self.contains_inf == other.contains_inf
	}
}

impl std::fmt::Debug for SornSet 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
	{
        let mut list = f.debug_list();
        for i in 0..self.len()
		{
            list.entry(&self.sets[i]);
        }
        list.finish()
    }
}