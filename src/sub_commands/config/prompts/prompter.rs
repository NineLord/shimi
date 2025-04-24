pub(super) const RETURN: &str = "↩ Return to previous menu";

pub struct Prompter<Theme: dialoguer::theme::Theme> {
	pub(super) theme: Theme
}

//#region Constructor
impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub const fn new(theme: Theme) -> Self {
		Self {
			theme
		}
	}
}
//#endregion

//#region Helpers with FuzzySelect/Select
pub enum Item<'a> {
	Str(&'a str),
	String(String),
}
pub enum Items<'a> {
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
pub enum Options<'a> {
	Item(Item<'a>, bool),
	Items(Items<'a>, Option<usize>),
}

macro_rules! insert_options_and_set_default {
	($prompter:ident, $options:ident) => ({
		let (index, found, new_prompter) = $options
			.iter()
			.fold((0, false, $prompter), |(mut index, mut found, mut $prompter), option| {
				match option {
					Options::Item(item, selected) => {
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
					Options::Items(items, selected) => {
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

#[derive(Default)]
pub struct Selection {
	pub vec_index: usize,
	pub options_index: usize,
}
pub enum SelectionReturn {
	Selection(Selection),
	Return,
}
pub(super) trait GetSelection {
	fn get_selection(&self, selected: usize) -> Selection;
}
impl GetSelection for Vec<Options<'_>> {
	fn get_selection(&self, mut selected: usize) -> Selection {
		for (index, options) in self.iter().enumerate() {
			match options {
				Options::Item(_, _) => {
					if selected == 0 {
						return Selection { vec_index: index, options_index: selected };
					}
					selected -= 1;
				},
				Options::Items(items, _) => {
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
}

macro_rules! from_repr {
	($enum:ident, $index:ident) => {
		$enum::from_repr(
			u8::try_from($index).expect("Enum not suppose to have more than u8::MAX variants")
		).expect("dialoguer::prompts::select::FuzzySelect insures only valid discriminant will be received")
	};
}
pub(crate) use from_repr;
//#endregion
