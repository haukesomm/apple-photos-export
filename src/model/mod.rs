pub mod asset;
pub mod album;
pub mod uti;

// TODO Does this trait really add any value?
pub trait FromDbModel<T> {
    fn from_db_model(model: &T) -> Result<Self, String>
        where Self: Sized;
}