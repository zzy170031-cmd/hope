pub trait PromptRenderer {
    type Input;
    type Output;
    type Error;

    fn render(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

pub trait LLMProvider {
    type Request;
    type Response;
    type Error;

    fn complete(&self, request: &Self::Request) -> Result<Self::Response, Self::Error>;
}

pub trait TextGenerationProvider {
    type Request;
    type Response;
    type Error;

    fn generate_text(&self, request: &Self::Request) -> Result<Self::Response, Self::Error>;
}

pub trait Validator {
    type Input;
    type Output;
    type Error;

    fn validate(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

pub trait Exporter {
    type Input;
    type Output;
    type Error;

    fn export(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}
