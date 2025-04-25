pub(super) const RETURN: &str = "↩ Return to previous menu";

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

// Options can be better:
// 1. Add macro like `vec!` for it.
// 2. Change it to state-machine, meaning forcing to call `set_selection`
//    when done inserting in order to pass it to the prompter.
pub struct Options<'a>(pub(super) Vec<AnyItem<'a>>);

impl Options<'_> {
	pub const fn new() -> Self {
		Self(Vec::new())
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self(Vec::with_capacity(capacity))
	}
}

impl <'a> Options<'a> {
	pub fn insert_str(mut self, item: &'a str) -> Self {
		self.0.push(AnyItem::Item(Item::Str(item), false));
		self
	}

	pub fn insert_string(mut self, item: String) -> Self {
		self.0.push(AnyItem::Item(Item::String(item), false));
		self
	}

	pub fn insert_str_refs(mut self, items: &'a [&'a str]) -> Self {
		self.0.push(AnyItem::Items(Items::StrRefs(items), None));
		self
	}

	pub fn insert_strings(mut self, items: &'a [String]) -> Self {
		self.0.push(AnyItem::Items(Items::Strings(items), None));
		self
	}

	pub fn insert_string_refs(mut self, items: &'a [&'a String]) -> Self {
		self.0.push(AnyItem::Items(Items::StringRefs(items), None));
		self
	}
}

impl Options<'_> {
	pub fn len(&self) -> usize {
		self.0.len()
	}
}

impl Options<'_> {
	pub(super) fn get_selection(&self, mut selected: usize) -> Selection {
		for (index, options) in self.0.iter().enumerate() {
			match options {
				AnyItem::Item(_, _) => {
					if selected == 0 {
						return Selection { vec_index: index, options_index: selected };
					}
					selected -= 1;
				},
				AnyItem::Items(items, _) => {
					let len = items.len();
					if selected < len {
						return Selection { vec_index: index, options_index: selected };
					}
					selected -= len;
				},
			}
		}
		unreachable!("The given selected {} is out of bound of self", selected)
	}

	pub fn set_selection(mut self, selected: &Selection) -> Self {
		let options = self.0.get_mut(selected.vec_index)
			.expect("The given selection index ins't valid");
		match options {
			AnyItem::Item(_, select) => *select = true,
			AnyItem::Items(_, select) => { let _ = select.insert(selected.options_index); },
		}
		self
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
		let (index, found, new_prompter) = $options.0
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
