use std::cell::RefCell;
use std::f64::INFINITY;
use std::f64::NEG_INFINITY;
use std::collections::HashMap;
use std::rc::Rc;

use crate::sornset::SornSet;
use crate::sornset::SornValue;

/* Change the type to u64 if u need more bits */
pub type SornBitsType = u128;

pub fn sorn_max_bits() -> usize
{
	(std::mem::size_of::<SornBitsType>() * 8).try_into().unwrap()
}

#[derive(Debug)]
pub struct Sorn
{
	pub bits: SornBitsType,
	pub sorn_set: Rc<RefCell<SornSet>>,
}

#[derive(Debug)]
pub enum SornErrors
{
	NotInRange,
	DifferentSornSets,
}

impl Sorn
{
	pub fn default() -> Self
	{
		Sorn
		{
			bits: 0,
			sorn_set: Rc::new(RefCell::new(SornSet::default())),
		}
	}

	pub fn new(set: Rc<RefCell<SornSet>>) -> Self
	{
		/* Create and return the Sorntype */
		Sorn
		{
			bits: 0,
			sorn_set: set
		}
	}

	pub fn new_array<const N: usize>(custom_set: Rc<RefCell<SornSet>>) -> [Self; N]
	{
		let arr: [Sorn; N] = core::array::from_fn(|_|
		{
			Sorn::new(custom_set.clone())
		});

		return arr;
	}

	pub fn from_sornvalue(set: Rc<RefCell<SornSet>>, value: SornValue) -> Sorn
	{
		let mut result = Sorn::new(set.clone());
		let bits = Sorn::sorn_to_bits(set.clone(), &value);

		result.set_bits(bits).unwrap();

		return result;
	}

	pub fn set_value(&mut self, value: SornValue)
	{
		let mut pos = 0;

		for (i, set) in self.sorn_set.borrow().sets.iter().enumerate()
		{
			if *set == value
			{
				pos = 1 << i;
				break;
			}
		}

		self.set_bits(pos);
	}

	pub fn set_bits(&mut self, bits: SornBitsType) -> Result<(), SornErrors>
	{
		if bits.leading_zeros() < (sorn_max_bits() - self.sorn_set.borrow().len()) as u32
		{
			return Err(SornErrors::NotInRange);
		}

		self.bits = bits;

		Ok(())
	}

	pub fn contains(&self, value: SornValue) -> bool
	{
		for (i, set) in self.sorn_set.borrow().sets.iter().enumerate()
		{
			if value == *set && (((1 << i) & self.bits) > 0)
			{
				return true;
			}
		}

		return false;
	}

	pub fn fit_contains(&self, value: SornValue) -> bool
	{
		let bit = Sorn::sorn_to_bits(self.sorn_set.clone(), &value);
		(self.bits & bit) > 0
	}

	pub fn get_ranges(&self) -> SornSet
	{
		let mut valid_ranges = SornSet::default();
		let sorn_set = self.sorn_set.borrow();
		let mut bits = self.bits;

		for i in 0..sorn_set.len()
		{
			if bits & 1 > 0
			{
				valid_ranges.push(sorn_set.get(i));
			}
			bits >>= 1;
			
			if bits == 0 { break; }
		}

		return valid_ranges;
	}

	pub fn get_min_range(&self) -> Option<SornValue>
	{
		if self.bits == 0
		{
			return None;
		}

		return Some(self.get_ranges().get(0));
	}

	pub fn get_max_range(&self) -> Option<SornValue>
	{
		let ranges = self.get_ranges();

		if ranges.len() == 0
		{
			return None;
		}

		return Some(ranges.get(ranges.len()-1));
	}

	pub fn to_sornvalue(&self) -> SornValue
	{
		let ranges = self.get_ranges();

		return ranges.get(0);
	}

	pub fn sorn_to_bits(sorn_set: Rc<RefCell<SornSet>>, value: &SornValue) -> SornBitsType
	{
		let start = std::time::Instant::now();
		let mut result = 0;

		for (i, item) in sorn_set.borrow().sets.iter().enumerate()
		{
			/* Check for Overlaps */
			if (value.is_exact() && item.is_exact()) && (value.get().unwrap() == item.get().unwrap()) ||
			(value.is_interval() && item.is_interval()) && (value.min() < item.max() && item.min() < value.max()) ||
			(value.is_open() && item.is_exact()) && (value.min() < item.get().unwrap() && value.max() > item.get().unwrap()) ||
			(value.is_exact() && item.is_open()) && (item.min() < value.get().unwrap() && item.max() > value.get().unwrap()) ||
			(value.is_leftopen() && item.is_exact()) && (value.min() < item.get().unwrap() && value.max() >= item.get().unwrap()) ||
			(value.is_exact() && item.is_leftopen()) && (item.min() < value.get().unwrap() && item.max() >= value.get().unwrap()) ||
			(value.is_rightopen() && item.is_exact()) && (value.min() <= item.get().unwrap() && value.max() > item.get().unwrap()) ||
			(value.is_exact() && item.is_rightopen()) && (item.min() <= value.get().unwrap() && item.max() > value.get().unwrap())
			{
				result |= 1 << i;
			}
			else if item.is_pminf() && value.is_pminf() && sorn_set.borrow().contains_inf
			{
				result |= 1 << 0;
			}
		}

		return result;
	}

	/* TODO only supports normal ranges and exacts */
	pub fn pow(&mut self, power: i32) -> Sorn
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());
		let mut result: SornBitsType = 0;

		if self.sorn_set.borrow().precomputed_pow.contains_key(&self.bits)
		{
			let result = 
			{
				let mut_set = self.sorn_set.borrow_mut();
				*mut_set.precomputed_pow.get(&self.bits).unwrap()
			};
			let _ = sorn.set_bits(result);
			return sorn;
		}
	
		for val in &self.get_ranges().sets 
		{
			let new_val = match val 
			{
				SornValue::Exact(v) => 
				{
					SornValue::Exact(v.powi(power))
				}
	
				SornValue::Open((start, end)) => 
				{
					let a = start.powi(power);
					let b = end.powi(power);

					if a > b 
					{
						SornValue::Open((b, a))
					}
					else
					{
						SornValue::Open((a, b))
					}
				}
	
				SornValue::OpenLeft((start, end)) => 
				{
					let a = start.powi(power);
					let b = end.powi(power);

					if a > b 
					{
						SornValue::OpenRight((b, a))
					}
					else
					{
						SornValue::OpenLeft((a, b))
					}
				}
	
				SornValue::OpenRight((start, end)) => 
				{
					let a = start.powi(power);
					let b = end.powi(power);

					if a > b 
					{
						SornValue::OpenLeft((b, a))
					}
					else
					{
						SornValue::OpenRight((a, b))
					}
				}
	
				SornValue::PlusMinusInf => 
				{
					SornValue::PlusMinusInf
				}
	
				SornValue::Empty => SornValue::Empty,
			};
	
			result |= Self::sorn_to_bits(self.sorn_set.clone(), &new_val);
		}

		self.sorn_set.borrow_mut().precomputed_pow.insert(self.bits, result);
	
		let _ = sorn.set_bits(result);
		sorn
	}	

	pub fn abs(&mut self) -> Sorn
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());
		let mut result: SornBitsType = 0;

		for val in &self.get_ranges().sets 
		{
			let new_val = match val 
			{
				SornValue::Exact(v) => 
				{
					SornValue::Exact(v.abs())
				}
	
				SornValue::Open((start, end)) => 
				{
					let a = start.abs();
					let b = end.abs();

					if a > b 
					{
						SornValue::Open((b, a))
					}
					else
					{
						SornValue::Open((a, b))
					}
				}
	
				SornValue::OpenLeft((start, end)) => 
				{
					let a = start.abs();
					let b = end.abs();

					if a > b 
					{
						SornValue::OpenRight((b, a))
					}
					else
					{
						SornValue::OpenLeft((a, b))
					}
				}
	
				SornValue::OpenRight((start, end)) => 
				{
					let a = start.abs();
					let b = end.abs();

					if a > b 
					{
						SornValue::OpenLeft((b, a))
					}
					else
					{
						SornValue::OpenRight((a, b))
					}
				}
	
				SornValue::PlusMinusInf => 
				{
					SornValue::PlusMinusInf
				}
	
				SornValue::Empty => SornValue::Empty,
			};
	
			result |= Self::sorn_to_bits(self.sorn_set.clone(), &new_val);
		}

		let _ = sorn.set_bits(result);
		sorn
	}

	pub fn negate(&mut self) -> Sorn
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());
		let mut result: SornBitsType = 0;

		for val in &self.get_ranges().sets 
		{
			let new_val = match val 
			{
				SornValue::Exact(v) => 
				{
					SornValue::Exact(-v)
				}
	
				SornValue::Open((start, end)) => 
				{
					let a = -start;
					let b = -end;

					if a > b 
					{
						SornValue::Open((b, a))
					}
					else
					{
						SornValue::Open((a, b))
					}
				}
	
				SornValue::OpenLeft((start, end)) => 
				{
					let a = -start;
					let b = -end;

					if a > b 
					{
						SornValue::OpenRight((b, a))
					}
					else
					{
						SornValue::OpenLeft((a, b))
					}
				}
	
				SornValue::OpenRight((start, end)) => 
				{
					let a = -start;
					let b = -end;

					if a > b 
					{
						SornValue::OpenLeft((b, a))
					}
					else
					{
						SornValue::OpenRight((a, b))
					}
				}
	
				SornValue::PlusMinusInf => 
				{
					SornValue::PlusMinusInf
				}
	
				SornValue::Empty => SornValue::Empty,
			};
	
			result |= Self::sorn_to_bits(self.sorn_set.clone(), &new_val);
		}

		let _ = sorn.set_bits(result);
		sorn
	}

	fn checked_op(&mut self, operand: &Self, operation: &str) -> Option<SornErrors>
	{
		if self.sorn_set != operand.sorn_set
		{
			return Some(SornErrors::DifferentSornSets);
		}

		let self_ranges = self.get_ranges();
		let operand_ranges = operand.get_ranges();

		let mut result: SornBitsType = 0;

		if operation == "add"
		{
			if self.sorn_set.borrow().precomputed_add.contains_key(&(self.bits, operand.bits))
			{
				let mut result: SornBitsType = 0;
				{
					let mut_set = self.sorn_set.borrow_mut();
					result = *mut_set.precomputed_add.get(&(self.bits, operand.bits)).unwrap();
				}
	
				let _ = self.set_bits(result);
				return None;
			}
		}

		if operation == "sub"
		{
			if self.sorn_set.borrow().precomputed_sub.contains_key(&(self.bits, operand.bits))
			{
				let mut result: SornBitsType = 0;
				{
					let mut_set = self.sorn_set.borrow_mut();
					result = *mut_set.precomputed_sub.get(&(self.bits, operand.bits)).unwrap();
				}
	
				let _ = self.set_bits(result);
				return None;
			}
		}

		if operation == "mul"
		{
			if self.sorn_set.borrow().precomputed_mul.contains_key(&(self.bits, operand.bits))
			{
				let mut result: SornBitsType = 0;
				{
					let mut_set = self.sorn_set.borrow_mut();
					result = *mut_set.precomputed_mul.get(&(self.bits, operand.bits)).unwrap();
				}
	
				let _ = self.set_bits(result);
				return None;
			}
		}

		if operation == "div"
		{
			if self.sorn_set.borrow().precomputed_div.contains_key(&(self.bits, operand.bits))
			{
				let mut result: SornBitsType = 0;
				{
					let mut_set = self.sorn_set.borrow_mut();
					result = *mut_set.precomputed_div.get(&(self.bits, operand.bits)).unwrap();
				}
	
				let _ = self.set_bits(result);
				return None;
			}
		}

		/* Handle plus minus inf special case */
		if self.contains(SornValue::PlusMinusInf) && operand.contains(SornValue::PlusMinusInf) 
		{
			let _ = self.set_bits(!0);
		}
		else if self.contains(SornValue::PlusMinusInf) || operand.contains(SornValue::PlusMinusInf) 
		{
			let mut result = 0;
			for i in 0..self.sorn_set.borrow().len()
			{
				result |= 1 << i;
			}

			let _ = self.set_bits(result);
		}

		/* Handle normal cases */
		for sorn1 in &self_ranges.sets
		{
			for sorn2 in &operand_ranges.sets
			{
				let (a, b) = match operation
				{
					"add" => 
					{
						let a = sorn1.min() + sorn2.min();
						let b = sorn1.min() + sorn2.max();
						let c = sorn1.max() + sorn2.min();
						let d = sorn1.max() + sorn2.max();
						let min = f64::min(f64::min(a, b), f64::min(c, d));
						let max = f64::max(f64::max(a, b), f64::max(c, d));

						(min, max)
					}
					"sub" =>
					{
						let a = sorn1.min() - sorn2.min();
						let b = sorn1.min() - sorn2.max();
						let c = sorn1.max() - sorn2.min();
						let d = sorn1.max() - sorn2.max();
						let min = f64::min(f64::min(a, b), f64::min(c, d));
						let max = f64::max(f64::max(a, b), f64::max(c, d));

						(min, max)
					}
					"mul" => 
					{
						let a = sorn1.min() * sorn2.min();
						let b = sorn1.min() * sorn2.max();
						let c = sorn1.max() * sorn2.min();
						let d = sorn1.max() * sorn2.max();
						let min = f64::min(f64::min(a, b), f64::min(c, d));
						let max = f64::max(f64::max(a, b), f64::max(c, d));

						(min, max)
					}
					"div" => 
					{
						let a = sorn1.min() / sorn2.min();
						let b = sorn1.min() / sorn2.max();
						let c = sorn1.max() / sorn2.min();
						let d = sorn1.max() / sorn2.max();
						let min = f64::min(f64::min(a, b), f64::min(c, d));
						let max = f64::max(f64::max(a, b), f64::max(c, d));

						(min, max)
					}
					_ => (0.0, 0.0)
				};

				/* Exact numbers always equal exact ones */
				if (sorn1.is_exact() && sorn2.is_exact()) || (operation == "mul" && (a, b) == (0.0, 0.0))
				{
					result |= Self::sorn_to_bits(self.sorn_set.clone(), &SornValue::Exact(a));
				}
				/* If one of the Numbers is Open, both become open after any operation */
				else if sorn1.is_open() || sorn2.is_open()
				{
					result |= Self::sorn_to_bits(self.sorn_set.clone(), &SornValue::Open((a, b)));
				}
				/* If both are leftopen or one exact and the other leftopen its always results in OpenLeft */
				else if (sorn1.is_leftopen() && sorn2.is_leftopen()) || (sorn1.is_leftopen() && sorn2.is_exact())
				|| (sorn1.is_exact() && sorn2.is_leftopen())
				{
					result |= Self::sorn_to_bits(self.sorn_set.clone(), &SornValue::OpenLeft((a, b)));
				}
				/* If both are rightopen or one exact and the other rightopen its always results in OpenRight */
				else if (sorn1.is_rightopen() && sorn2.is_rightopen()) || (sorn1.is_rightopen() && sorn2.is_exact())
				|| (sorn1.is_exact() && sorn2.is_rightopen())
				{
					result |= Self::sorn_to_bits(self.sorn_set.clone(), &SornValue::OpenRight((a, b)));
				}
				/* If they have opposite open directions then the result becomes Open */
				else if (sorn1.is_leftopen() && sorn2.is_rightopen()) || (sorn1.is_rightopen() && sorn2.is_leftopen())
				{
					result |= Self::sorn_to_bits(self.sorn_set.clone(), &SornValue::Open((a, b)));
				}
				else if (a == INFINITY && b == INFINITY) || (a == NEG_INFINITY && b == NEG_INFINITY)
				{
					result |= Self::sorn_to_bits(self.sorn_set.clone(), &SornValue::PlusMinusInf);
				}
			}
		}

		if operation == "add"
		{
			self.sorn_set.borrow_mut().precomputed_add.insert((self.bits, operand.bits), result);
			self.sorn_set.borrow_mut().precomputed_add.insert((operand.bits, self.bits), result);
		}

		if operation == "sub"
		{
			self.sorn_set.borrow_mut().precomputed_sub.insert((self.bits, operand.bits), result);
			self.sorn_set.borrow_mut().precomputed_sub.insert((operand.bits, self.bits), result);
		}

		if operation == "mul"
		{
			self.sorn_set.borrow_mut().precomputed_mul.insert((self.bits, operand.bits), result);
			self.sorn_set.borrow_mut().precomputed_mul.insert((operand.bits, self.bits), result);
		}

		if operation == "div"
		{
			self.sorn_set.borrow_mut().precomputed_div.insert((self.bits, operand.bits), result);
			self.sorn_set.borrow_mut().precomputed_div.insert((operand.bits, self.bits), result);
		}
		
		let res = self.set_bits(result);
		//println!("{}", self.to_string());

		return None;
	}

	pub fn checked_add(&mut self, addend: &Self) -> Option<SornErrors>
	{
		Self::checked_op(self, addend, "add")
	}

	pub fn checked_sub(&mut self, addend: &Self) -> Option<SornErrors>
	{
		Self::checked_op(self, addend, "sub")
	}

	pub fn checked_mul(&mut self, addend: &Self) -> Option<SornErrors>
	{
		Self::checked_op(self, addend, "mul")
	}

	pub fn checked_div(&mut self, addend: &Self) -> Option<SornErrors>
	{
		Self::checked_op(self, addend, "div")
	}
}

impl std::ops::Neg for Sorn
{
	type Output = Sorn;

	fn neg(mut self) -> Self::Output 
	{
		self.negate()
	}
}

/* Addition Operator */
impl std::ops::Add for Sorn
{
	type Output = Sorn;

	fn add(self, rhs: Self) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_add(&rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Add for &Sorn
{
	type Output = Sorn;

	fn add(self, rhs: Self) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_add(rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Add<&Sorn> for Sorn
{
	type Output = Sorn;

	fn add(self, rhs: &Sorn) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_add(rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Add<Sorn> for &Sorn
{
	type Output = Sorn;

	fn add(self, rhs: Sorn) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_add(&rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::AddAssign<&Sorn> for Sorn
{
	fn add_assign(&mut self, rhs: &Sorn) 
	{
		let res = self.checked_add(&rhs);

		if res.is_some()
		{
			self.bits = 0;
		}
	}
}

/* Subtraction Operator */
impl std::ops::Sub for Sorn
{
	type Output = Sorn;

	fn sub(self, rhs: Self) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_sub(&rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Sub for &Sorn
{
	type Output = Sorn;

	fn sub(self, rhs: Self) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_sub(rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Sub<&Sorn> for Sorn
{
	type Output = Sorn;

	fn sub(self, rhs: &Sorn) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_sub(rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Sub<Sorn> for &Sorn
{
	type Output = Sorn;

	fn sub(self, rhs: Sorn) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_sub(&rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::SubAssign<&Sorn> for Sorn
{
	fn sub_assign(&mut self, rhs: &Sorn) 
	{
		let res = self.checked_sub(&rhs);

		if res.is_some()
		{
			self.bits = 0;
		}
	}
}

/* Multiplication Operator */
impl std::ops::Mul for Sorn
{
	type Output = Sorn;

	fn mul(self, rhs: Self) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_mul(&rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Mul for &Sorn
{
	type Output = Sorn;

	fn mul(self, rhs: Self) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_mul(rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Mul<&Sorn> for Sorn
{
	type Output = Sorn;

	fn mul(self, rhs: &Sorn) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_mul(rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Mul<Sorn> for &Sorn
{
	type Output = Sorn;

	fn mul(self, rhs: Sorn) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_mul(&rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::MulAssign<&Sorn> for Sorn
{
	fn mul_assign(&mut self, rhs: &Sorn) 
	{
		let res = self.checked_mul(&rhs);

		if res.is_some()
		{
			self.bits = 0;
		}
	}
}

/* Division Operator */
impl std::ops::Div for Sorn
{
	type Output = Sorn;

	fn div(self, rhs: Self) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_div(&rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Div for &Sorn
{
	type Output = Sorn;

	fn div(self, rhs: Self) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_div(rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Div<&Sorn> for Sorn
{
	type Output = Sorn;

	fn div(self, rhs: &Sorn) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_div(rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::Div<Sorn> for &Sorn
{
	type Output = Sorn;

	fn div(self, rhs: Sorn) -> Self::Output 
	{
		let mut sorn = Sorn::new(self.sorn_set.clone());

		let res1 = sorn.set_bits(self.bits);
		let res2 = sorn.checked_div(&rhs);

		if res1.is_err() || res2.is_some()
		{
			sorn.bits = 0;
		}

		return sorn;
	}
}

impl std::ops::DivAssign<&Sorn> for Sorn
{
	fn div_assign(&mut self, rhs: &Sorn) 
	{
		let res = self.checked_div(&rhs);

		if res.is_some()
		{
			self.bits = 0;
		}
	}
}

impl std::clone::Clone for Sorn
{
	fn clone(&self) -> Self 
	{
		Self { bits: self.bits.clone(), sorn_set: self.sorn_set.clone() }
	}
}

impl std::string::ToString for Sorn
{
	fn to_string(&self) -> String 
	{
		let range = self.get_ranges();

		format!("Bits: {:b} | Range: {:?}", self.bits, range)
	}
}

impl Sorn
{
	pub fn to_string_hex(&self) -> String
	{
		let range = self.get_ranges();

		format!("{:X}", self.bits)
	}

	pub fn to_string_hex_full(&self) -> String
	{
		let range = self.get_ranges();

		format!("{:X} | Range: {:?}", self.bits, range)
	}

	pub fn to_string_nobits(&self) -> String
	{
		let range = self.get_ranges();

		format!("{:?}", range)
	}

	pub fn to_string_compact(&self) -> String
	{
		let range = self.get_ranges();

		format!("{} to {}", range.get_min_range().unwrap().min(), range.get_max_range().unwrap().max())
	}
}

impl std::cmp::PartialEq for Sorn
{
	fn eq(&self, other: &Self) -> bool 
	{
		(self.sorn_set == other.sorn_set) && (self.bits == other.bits)
	}
}

/* Testing */
#[cfg(test)]
mod tests 
{
    use crate::sorntable_gen;

    use super::*;
	use super::SornValue::*;

	#[test]
	fn test_new()
	{
		
	}

	#[test]
	fn test_sorn_to_bits_positive()
	{
		/* 
		1: [0] 
		10: (0, 1)
		100: [1]
		*/
		let mut set = Rc::new(RefCell::new(SornSet::new(0.0, 1.0, 1.0, false)));
		let mut sorn = Sorn::new(set.clone());
		
		/* Intervals */
		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Open((0.0, 1.0)));
		assert_eq!(value, 0b10);

		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Open((0.0, 2.0)));
		assert_eq!(value, 0b110);
		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Open((0.0, 1.245345)));
		assert_eq!(value, 0b110);

		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Open((-1.0, 2.0)));
		assert_eq!(value, 0b111);
		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Open((-0.000000001, 2.0)));
		assert_eq!(value, 0b111);

		/* Exacts */
		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Exact(0.0));
		assert_eq!(value, 0b001);
		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Exact(1.0));
		assert_eq!(value, 0b100);
		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Exact(0.5));
		assert_eq!(value, 0b010);
	}

	#[test]
	fn test_sorn_to_bits_negative()
	{
		/* 
		1: [-2] 
		10: (-2, -1)
		100: [-1]
		*/
		let set = Rc::new(RefCell::new(SornSet::new(-2.0, -1.0, 1.0, false)));
		let mut sorn = Sorn::new(set.clone());
	
		/* Intervals */
		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Open((-2.0, -1.0)));
		assert_eq!(value, 0b10);

		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Open((-2.0, 2.0)));
		assert_eq!(value, 0b110);

		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Open((-1.0, 2.0)));
		assert_eq!(value, 0b000);

		/* Exacts */
		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Exact(-2.0));
		assert_eq!(value, 0b001);

		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Exact(-1.0));
		assert_eq!(value, 0b100);

		let value = Sorn::sorn_to_bits(sorn.sorn_set.clone(), &Exact(-1.5));
		assert_eq!(value, 0b010);
	}

	#[test]
	fn test_custom_new()
	{
		
	}

	#[test]
	fn test_set_number()
	{
		
	}

	#[test]
	fn test_set_bits()
	{
		
	}

	#[test]
	fn test_posneg_noinf_add()
	{
		/* CSV Format */
		let expected = 	",1,10,100,1000,10000,\n\
						1,0,0,1,10,100,\n\
						10,0,11,10,1110,1000,\n\
						100,1,10,100,1000,10000,\n\
						1000,10,1110,1000,11000,0,\n\
						10000,100,1000,10000,0,0,\n\
						";

		let set = Rc::new(RefCell::new(SornSet::new(-1.0, 1.0, 1.0, false)));
		let sorn1 = Sorn::new(set.clone());
		let table = sorntable_gen::gen_table(sorn1.sorn_set.clone(), "add");

		println!("{}", table.to_csv());
		println!("{}", expected);

		assert_eq!(table.to_csv(), expected);
	}

	#[test]
	fn test_neg_inf_add()
	{
		let expected = ",1,10,100,1000,10000,100000,\n\
						1,1,111111,111111,111111,111111,111111,\n\
						10,111111,10,10,10,10,111110,\n\
						100,111111,10,10,10,100,111000,\n\
						1000,111111,10,10,1110,1000,111000,\n\
						10000,111111,10,100,1000,10000,100000,\n\
						100000,111111,111110,111000,111000,100000,100000,\n\
						";

		let set = Rc::new(RefCell::new(SornSet::new(-1.0, 0.0, 1.0, true)));
		let sorn1 = Sorn::new(set.clone());
		let table = sorntable_gen::gen_table(sorn1.sorn_set.clone(), "add");

		println!("{}", table.to_csv());
		println!("{}", expected);

		assert_eq!(table.to_csv(), expected);
	}
}