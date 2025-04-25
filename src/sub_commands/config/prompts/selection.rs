use state_shift::{type_state, impl_state};

const RETURN: &str = "↩ Return to previous menu";

//#region Options
#[allow(dead_code)]
pub(super) enum Item<'a> {
	Str(&'a str),
	String(String),
}
#[allow(dead_code)]
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
#[allow(dead_code)]
pub(super) enum AnyItem<'a> {
	Item(Item<'a>, bool),
	Items(Items<'a>, Option<usize>),
}

#[type_state(
    states = (Inserting, SelectingReturn, Selected, SelectedReturn, ReSelecting), // defines the available states
    slots = (Inserting) // defines how many concurrent states will be there, and the initial values for these states
)]
pub struct Options<'a> {
	pub(super) items: Vec<AnyItem<'a>>,
	is_return: bool,
	selected: bool,
}

#[impl_state]
impl <'a> Options<'a> {
	#[require(Inserting)] // require the default state for the constructor
	pub fn with_capacity(capacity: usize) -> Self {
		Options {
			items: Vec::with_capacity(capacity),
			is_return: false,
			selected: false,
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
	#[switch_to(SelectingReturn)]
	pub fn insert_return(self) -> Options<'a> {
		Options {
			selected: self.selected,
			items: self.insert_str(RETURN).items,
			is_return: true,
		}
	}

	#[require(Inserting)]
	#[switch_to(Selected)]
	pub fn set_selection(mut self, selected: &Selection) -> Options<'a> {
		Options::set_selection_on_options(&mut self.items, selected);
		Options {
			items: self.items,
			is_return: self.is_return,
			selected: true,
		}
	}

	#[require(SelectingReturn)]
	#[switch_to(SelectedReturn)]
	pub fn set_selection(mut self, selected: &SelectionReturn) -> Options<'a> {
		Options::set_selection_on_options_return(&mut self.items, selected);
		Options {
			items: self.items,
			is_return: self.is_return,
			selected: true,
		}
	}

	#[require(Selected)]
	pub fn re_set_selection(mut self, selected: &Selection) -> Options<'a> {
		Options::clear_prev_selection(&mut self.items);
		Options::set_selection_on_options(&mut self.items, selected);
		Options {
			items: self.items,
			is_return: self.is_return,
			selected: true,
		}
	}

	#[require(SelectedReturn)]
	pub fn re_set_selection(mut self, selected: &SelectionReturn) -> Options<'a> {
		Options::clear_prev_selection(&mut self.items);
		Options::set_selection_on_options_return(&mut self.items, selected);
		Options {
			items: self.items,
			is_return: self.is_return,
			selected: true,
		}
	}
}

impl Options<'_, Selected> {
	pub(super) fn get_selection(&self, selected: usize) -> Selection {
		Options::get_selection_options(&self.items, selected, false)
			.expect("The given selected is out of bound of self")
	}
}

impl Options<'_, SelectedReturn> {
	pub(super) fn get_selection(&self, selected: usize) -> SelectionReturn {
		Options::get_selection_options(&self.items, selected, true)
			.map_or(SelectionReturn::Return, SelectionReturn::Selection)
	}
}

impl <'a> Options<'a> {
	fn get_selection_options(options: &Vec<AnyItem<'_>>, mut selected: usize, is_skip_last: bool) -> Option<Selection> {
		// Have to resort to macro, due to not being able to pass any iterator to a function.
		// Also it's `iter.for_each` doesn't allow to break early.
		macro_rules! find_selection {
			($iter:expr) => ({
				for (index, options) in $iter {
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
			});
		}

		let iter = options.iter().enumerate();
		if is_skip_last {
			find_selection!(iter.take(options.len() -1));
		} else {
			find_selection!(iter);
		}
		None
	}

	fn clear_prev_selection(options: &mut [AnyItem<'a>]) {
		for options in options {
			match options {
				AnyItem::Item(_, selected) => {
					if *selected {
						*selected = false;
						break;
					}
				},
				AnyItem::Items(_, selected) => {
					if selected.is_some() {
						selected.take();
						break;
					}
				},
			}
		}
	}

	fn set_selection_on_options(options: &mut [AnyItem<'a>], selected: &Selection) {
		let options = options.get_mut(selected.vec_index)
			.expect("The given selection index ins't valid");
		match options {
			AnyItem::Item(_, select) => *select = true,
			AnyItem::Items(_, select) => *select = Some(selected.options_index),
		}
	}

	fn set_selection_on_options_return(options: &mut [AnyItem<'a>], selected: &SelectionReturn) {
		match selected {
			SelectionReturn::Selection(selection) => Options::set_selection_on_options(options, selection),
			SelectionReturn::Return => {
				let last = options.last_mut()
					.expect("Due to being in SelectingReturn, there has to be last item which is return");
				if let AnyItem::Item(_, select) = last {
					*select = true;
				} else {
					unreachable!("Due to being in SelectingReturn, the last item has to be return which is AnyItem::Item");
				}
			},
		}
	}
}
//#endregion

//#region Selection
#[allow(dead_code)]
#[derive(Default)]
pub struct Selection {
	pub vec_index: usize,
	pub options_index: usize,
}
#[allow(dead_code)]
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
