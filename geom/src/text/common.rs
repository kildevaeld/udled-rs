use udled::{tokenizers::AsciiWhiteSpace, AsChar, Buffer, Reader, TokenizerExt};

pub fn ws<'input, B>(reader: &mut Reader<'_, 'input, B>) -> udled::Result<()>
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    reader.eat(AsciiWhiteSpace.many())?;
    Ok(())
}
