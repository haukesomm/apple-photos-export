pub mod asset;
pub mod album;
pub mod uti;

pub trait FromDbModel<T> {
    fn from_db_model(model: T) -> Result<Self, String>
        where Self: Sized;
}