pub trait LexiconIfce<H>
where
    H: Copy + PartialEq,
{
    /// Returns number of skippable bytes at start of `text`.
    fn skippable_count(&self, text: &str) -> usize;
    /// Returns the longest literal match at start of `text`.
    fn longest_literal_match(&self, text: &str) -> Option<(H, usize)>;
    /// Returns the longest regular expression match at start of `text`.
    fn longest_regex_match(&self, text: &str) -> Option<Vec<(H, usize)>>;
    /// Returns the distance in bytes to the next valid content in `text`
    fn distance_to_next_valid_byte(&self, text: &str) -> Option<usize>;
}
