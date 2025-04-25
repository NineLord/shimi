use state_shift::{type_state, impl_state};

const RETURN: &str = "↩ Return to previous menu";

//#region Options
pub(super) enum Item<'a> {
	Str(&'a str),
	String(String),
}
pub(super) enum Items<'a> {
	StrRefs(&'a [&'a str]),
	Strings(&'a [String]),
	StringRefs(&'a [&'a String]),
}
impl Items<'_> {
	pub const fn len(&self) -> usize {
		match self {
			Items::StrRefs(items) => items.len(),
			Items::Strings(items) => items.len(),
			Items::StringRefs(items) => items.len(),
		}
	}
}
pub(super) enum AnyItem<'a> {
	Item(Item<'a>, bool),
	Items(Items<'a>, Option<usize>),
}

#[type_state(
    states = (Inserting, InsertingReturn, Done, DoneWithReturn), // defines the available states
    slots = (Inserting) // defines how many concurrent states will be there, and the initial values for these states
)]
pub struct Options<'a> {
	pub(super) items: Vec<AnyItem<'a>>,
	is_return: bool,
}

#[impl_state]
impl <'a> Options<'a> {
	#[require(Inserting)] // require the default state for the constructor
	pub fn with_capacity(capacity: usize) -> Self {
		Options {
			items: Vec::with_capacity(capacity),
			is_return: false,
		}
	}

	#[require(Inserting)]
	pub fn insert_str(mut self, item: &'a str) -> Self {
		self.items.push(AnyItem::Item(Item::Str(item), false));
		self
	}

	#[require(Inserting)]
	pub fn insert_string(mut self, item: String) -> Self {
		self.items.push(AnyItem::Item(Item::String(item), false));
		self
	}

	#[require(Inserting)]
	pub fn insert_str_refs(mut self, items: &'a [&'a str]) -> Self {
		self.items.push(AnyItem::Items(Items::StrRefs(items), None));
		self
	}

	#[require(Inserting)]
	pub fn insert_strings(mut self, items: &'a [String]) -> Self {
		self.items.push(AnyItem::Items(Items::Strings(items), None));
		self
	}

	#[require(Inserting)]
	pub fn insert_string_refs(mut self, items: &'a [&'a String]) -> Self {
		self.items.push(AnyItem::Items(Items::StringRefs(items), None));
		self
	}

	#[require(Inserting)]
	#[switch_to(InsertingReturn)]
	pub fn insert_return(self) -> Options<'a> {
		Options {
			items: self.insert_str(RETURN).items,
			is_return: true,
		}
	}

	#[require(Inserting)]
	#[switch_to(Done)]
	pub fn set_selection(mut self, selected: &Selection) -> Options<'a> {
		Options::set_selection_on_options(&mut self.items, selected);
		Options {
			items: self.items,
			is_return: self.is_return,
		}
	}

	#[require(InsertingReturn)]
	#[switch_to(DoneWithReturn)]
	pub fn set_selection(mut self, selected: &SelectionReturn) -> Options<'a> {
		match selected {
			SelectionReturn::Selection(selection) => Options::set_selection_on_options(&mut self.items, selection),
			SelectionReturn::Return => {
				let last = self.items.last_mut()
					.expect("Due to being in InsertingReturn, there has to be last item which is return");
				if let AnyItem::Item(_, select) = last {
					*select = true;
				} else {
					unreachable!("Due to being in InsertingReturn, the last item has to be return which is AnyItem::Item");
				}
			},
		}
		Options {
			items: self.items,
			is_return: self.is_return,
		}
	}
}

impl Options<'_, Done> {
	pub(super) fn get_selection(self, selected: usize) -> Selection {
		Options::get_selection_options(self.items, selected)
			.expect("The given selected is out of bound of self")
	}
}

impl Options<'_, DoneWithReturn> {
	pub(super) fn get_selection(mut self, selected: usize) -> SelectionReturn {
		let return_item = self.items.pop();
		debug_assert!(return_item.is_some(), "According to this state there must be a return as the last item");
		Options::get_selection_options(self.items, selected)
			.map_or(SelectionReturn::Return, SelectionReturn::Selection)
	}
}

impl <'a> Options<'a> {
	fn get_selection_options(options: Vec<AnyItem<'_>>, mut selected: usize) -> Option<Selection> {
		for (index, options) in options.into_iter().enumerate() {
			match options {
				AnyItem::Item(_, _) => {
					if selected == 0 {
						return Some(Selection { vec_index: index, options_index: selected });
					}
					selected -= 1;
				},
				AnyItem::Items(items, _) => {
					let len = items.len();
					if selected < len {
						return Some(Selection { vec_index: index, options_index: selected });
					}
					selected -= len;
				},
			}
		}
		None
	}

	fn set_selection_on_options(options: &mut [AnyItem<'a>], selected: &Selection) {
		let options = options.get_mut(selected.vec_index)
			.expect("The given selection index ins't valid");
		match options {
			AnyItem::Item(_, select) => *select = true,
			AnyItem::Items(_, select) => { let _ = select.insert(selected.options_index); },
		}
	}
}

impl Options<'_> {
	pub fn len(&self) -> usize {
		self.items.len()
	}
}
//#endregion

//#region Selection
#[derive(Default)]
pub struct Selection {
	pub vec_index: usize,
	pub options_index: usize,
}
pub enum SelectionReturn {
	Selection(Selection),
	Return,
}
impl Default for SelectionReturn {
	fn default() -> Self {
		Self::Selection(Selection::default())
	}
}
//#endregion

//#region Macros
macro_rules! insert_options_and_set_default {
	($prompter:ident, $options:ident) => ({
		let (index, found, new_prompter) = $options.items
			.iter()
			.fold((0, false, $prompter), |(mut index, mut found, mut $prompter), option| {
				match option {
					AnyItem::Item(item, selected) => {
						$prompter = match item {
							Item::Str(item) => $prompter.item(item),
							Item::String(item) => $prompter.item(item),
						};
						if !found {
							if *selected {
								found = true;
							} else {
								index += 1;
							}
						}
					},
					AnyItem::Items(items, selected) => {
						let (len, new_prompter) = match items {
							Items::StrRefs(items) => (items.len(), $prompter.items(items)),
							Items::Strings(items) => (items.len(), $prompter.items(items)),
							Items::StringRefs(items) => (items.len(), $prompter.items(items)),
						};
						$prompter = new_prompter;
						if !found {
							if let Some(selected) = selected {
								debug_assert!(*selected < len, "Given selection isn't within the bound of the given list");
								index += selected;
								found = true;
							} else {
								index += len;
							}
						}
					},
				}
				(index, found, $prompter)
			});
		$prompter = new_prompter;
		if found {
			$prompter = $prompter.default(index);
		}
	});
}
pub(super) use insert_options_and_set_default;

macro_rules! from_repr {
	($enum:ident, $index:expr) => {
		$enum::from_repr(
			u8::try_from($index).expect("Enum not suppose to have more than u8::MAX variants")
		).expect("dialoguer::prompts::select::FuzzySelect insures only valid discriminant will be received")
	};
}
pub(crate) use from_repr;
//#endregion
