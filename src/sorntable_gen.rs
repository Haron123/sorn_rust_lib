use std::{cell::RefCell, rc::Rc};

use crate::{sorn::Sorn, sornset::SornSet, sorn::SornBitsType};
pub struct SornTable
{
	sorn_sets: Rc<RefCell<SornSet>>,

	header: Vec<Sorn>,
	table_data: Vec<Vec<Sorn>>
}

/* possible operators: "add", "sub", "mul", "div" */
pub fn gen_table(sorn_sets: Rc<RefCell<SornSet>>, operator: &str) -> SornTable
{
    let mut sorns: Vec<Sorn> = Vec::new();

	let num_sets = sorn_sets.borrow().len();

	let mut header = vec![Sorn::default(); num_sets];
	let mut table_data = vec![vec![Sorn::default(); num_sets]; num_sets];

	/* Create a SORN for every bit */
    for i in 0..num_sets
    {
        let mut sorn = Sorn::new(sorn_sets.clone());

		/* Will always be valid in this case, so we dont need to check the return value */
        let _ = sorn.set_bits(1 << i);

        sorns.push(sorn);
    }

	/* Write every SORN generated before in the header as bits */
    for i in 0..num_sets
    {
        header[i] = sorns[i].clone();
    }

	/* Write the Tabledata */
    for i in 0..num_sets
    {
        for j in 0..num_sets
        {
			let cur;

			match operator
			{
				"add" => cur = &sorns[i] + &sorns[j],
				"sub" => cur = &sorns[i] - &sorns[j],
				"mul" => cur = &sorns[i] * &sorns[j],
				"div" => cur = &sorns[i] / &sorns[j],

				_ => panic!("Tried to generate SORN Table without valid operator, use 'add', 'sub', 'mul' or 'div'")
			}

            table_data[j][i] = cur; 
        }
    }

    SornTable
	{
		sorn_sets: sorn_sets.clone(),
		header,
		table_data,
	}
}

impl SornTable
{
	pub fn to_csv_bits(&self) -> String
	{
		let mut result: String = "".to_owned();
		let n = self.sorn_sets.borrow().len();

		/* Add the Row Header */
		result.push(',');
		for item in &self.header
		{
			result.push_str(&format!("{:0n$b},", item.bits));
		}
		result.push('\n');

		/* Add the Column Header alongside the Tabledata */
		for (i, row) in self.table_data.iter().enumerate()
		{
			result.push_str(&format!("{:0n$b},", self.header[i].bits));

			for col in row
			{
				result.push_str(&format!("{:0n$b},", col.bits));
			}

			result.push('\n');
		}

		return result;
	}

	pub fn to_csv_intervals(&self) -> String
	{
		let mut result: String = "".to_owned();
		let n = self.sorn_sets.borrow().len();

		/* Add the Row Header */
		result.push(',');
		for item in &self.header
		{
			result.push_str(&format!("{},", item.to_string_compact()));
		}
		result.push('\n');

		/* Add the Column Header alongside the Tabledata */
		for (i, row) in self.table_data.iter().enumerate()
		{
			result.push_str(&format!("{},", self.header[i].to_string_compact()));

			for col in row
			{
				result.push_str(&format!("{},", col.to_string_compact()));
			}

			result.push('\n');
		}

		return result;
	}
}

impl std::string::ToString for SornTable
{
	fn to_string(&self) -> String 
	{
		let mut result: String = "".to_owned();

		/* Add the Set */
		result.push_str(&format!("Sorn Set: {:?}\n", self.sorn_sets.borrow().sets));

		/* Add the Row Header */
		result.push_str("\t|\t");
		for item in &self.header
		{
			result.push_str(&format!("{:b}\t|\t", item.bits));
		}
		result.push('\n');
		result.push_str(&"-".repeat(self.header.len() * 20));
		result.push('\n');

		/* Add the Column Header alongside the Tabledata */
		for (i, row) in self.table_data.iter().enumerate()
		{
			result.push_str(&format!("{:b}\t|\t", self.header[i].bits));

			for col in row
			{
				result.push_str(&format!("{:b}\t|\t", col.bits));
			}

			result.push('\n');
		}

		return result;
	}
}
