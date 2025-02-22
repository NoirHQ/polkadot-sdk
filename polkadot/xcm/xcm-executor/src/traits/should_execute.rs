// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use core::result::Result;
use frame_support::traits::ProcessMessageError;
use xcm::latest::{Instruction, Location, Weight, XcmHash};

/// Properties of an XCM message and its imminent execution.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Properties {
	/// The amount of weight that the system has determined this
	/// message may utilize in its execution. Typically non-zero only because of prior fee
	/// payment, but could in principle be due to other factors.
	pub weight_credit: Weight,
	/// The identity of the message, if one is known. If left as `None`, then it will generally
	/// default to the hash of the message which may be non-unique.
	pub message_id: Option<XcmHash>,
}

/// Trait to determine whether the execution engine should actually execute a given XCM.
///
/// Can be amalgamated into a tuple to have multiple trials. If any of the tuple elements returns
/// `Ok(())`, the execution stops. Else, `Err(_)` is returned if all elements reject the message.
pub trait ShouldExecute {
	/// Returns `Ok(())` if the given `message` may be executed.
	///
	/// - `origin`: The origin (sender) of the message.
	/// - `instructions`: The message itself.
	/// - `max_weight`: The (possibly over-) estimation of the weight of execution of the message.
	/// - `properties`: Various pre-established properties of the message which may be mutated by
	///   this API.
	fn should_execute<RuntimeCall>(
		origin: &Location,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		properties: &mut Properties,
	) -> Result<(), ProcessMessageError>;
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl ShouldExecute for Tuple {
	fn should_execute<RuntimeCall>(
		origin: &Location,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		properties: &mut Properties,
	) -> Result<(), ProcessMessageError> {
		for_tuples!( #(
			let barrier = core::any::type_name::<Tuple>();
 			match Tuple::should_execute(origin, instructions, max_weight, properties) {
				Ok(()) => {
					tracing::trace!(
						target: "xcm::should_execute",
						?origin,
						?instructions,
						?max_weight,
						?properties,
						%barrier,
						"pass barrier",
					);
					return Ok(())
				},
				Err(error) => {
					tracing::trace!(
						target: "xcm::should_execute",
						?origin,
						?instructions,
						?max_weight,
						?properties,
						?error,
						%barrier,
						"did not pass barrier",
					);
				},
			}
		)* );

		Err(ProcessMessageError::Unsupported)
	}
}

/// Trait to determine whether the execution engine is suspended from executing a given XCM.
///
/// The trait method is given the same parameters as `ShouldExecute::should_execute`, so that the
/// implementer will have all the context necessary to determine whether or not to suspend the
/// XCM executor.
///
/// Can be chained together in tuples to have multiple rounds of checks. If all of the tuple
/// elements returns false, then execution is not suspended. Otherwise, execution is suspended
/// if any of the tuple elements returns true.
pub trait CheckSuspension {
	fn is_suspended<Call>(
		origin: &Location,
		instructions: &mut [Instruction<Call>],
		max_weight: Weight,
		properties: &mut Properties,
	) -> bool;
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl CheckSuspension for Tuple {
	fn is_suspended<Call>(
		origin: &Location,
		instruction: &mut [Instruction<Call>],
		max_weight: Weight,
		properties: &mut Properties,
	) -> bool {
		for_tuples!( #(
			if Tuple::is_suspended(origin, instruction, max_weight, properties) {
				return true
			}
		)* );

		false
	}
}

/// Trait to determine whether the execution engine should not execute a given XCM.
///
/// Can be amalgamated into a tuple to have multiple traits. If any of the tuple elements returns
/// `Err(ProcessMessageError)`, the execution stops. Else, `Ok(())` is returned if all elements
/// accept the message.
pub trait DenyExecution {
	/// Returns `Ok(())` if there is no reason to deny execution,
	/// while `Err(ProcessMessageError)` indicates there is a reason to deny execution.
	///
	/// - `origin`: The origin (sender) of the message.
	/// - `instructions`: The message itself.
	/// - `max_weight`: The (possibly over-) estimation of the weight of execution of the message.
	/// - `properties`: Various pre-established properties of the message which may be mutated by
	///   this API.
	fn deny_execution<RuntimeCall>(
		origin: &Location,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		properties: &mut Properties,
	) -> Result<(), ProcessMessageError>;
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl DenyExecution for Tuple {
	fn deny_execution<RuntimeCall>(
		origin: &Location,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		properties: &mut Properties,
	) -> Result<(), ProcessMessageError> {
		for_tuples!( #(
            let barrier = core::any::type_name::<Tuple>();
            match Tuple::deny_execution(origin, instructions, max_weight, properties) {
                Err(error) => {
                    tracing::error!(
                        target: "xcm::deny_execution",
                        ?origin,
                        ?instructions,
                        ?max_weight,
                        ?properties,
                        ?error,
                        %barrier,
                        "did not pass barrier",
                    );
                    return Err(error);
                },
				  Ok(())  => {
                    tracing::trace!(
                        target: "xcm::deny_execution",
                        ?origin,
                        ?instructions,
                        ?max_weight,
                        ?properties,
                        %barrier,
                        "pass barrier",
                    );
                },
            }
        )* );

		Ok(())
	}
}
