# Providers
Providers are what most other projects would call "integrations": they include
ways to augment notebooks via interactions with third-party resources.

## Important glossary
- **Provider**: a WASM (Web Assembly) plugin to be loaded in Fiberplane
  products. It knows how to connect to third-party resources and make them
  available to Fiberplane products following Fiberplane bindings.
- **Data Source**: a configured instance of a Provider. As a user, you connect
  your notebook to a Data Source.
- **Third-party resource**: the external service the Provider is written for.
- **Runtime**: an execution environment to run Providers in. The runtime
  provides host bindings that limit Providers access to their hosting
  environment. Currently the main runtimes are in the browser for Studio, and a
  Rust one used in the Proxy
  
## Overview of the protocol

``` mermaid
sequenceDiagram
    actor user as User
    participant studio as Fiberplane Studio
    participant ds as Data Source<br />(Provider Instance)
    participant tpr as Third-party Resource
    participant api as Fiberplane API
    user->>studio: Connect and create notebook
    critical get_supported_query_types
        studio->ds: Registration of providers and<br />list of SupportedQueryTypes
    end
    studio->>user: Show list of supported queries<br />as slashCommands
    user->>+studio: Choose a slashCommand to<br />activate integration
    studio->>studio: Find the correct data source<br />for a give query type
    critical get_supported_query_types
        studio->>+ds: Request with "provider_type.query_type"
    end
    ds->>-studio: List of QueryFields to fill
    studio->>-user: Show query fields
    user->>+studio: Fill query fields<br />(user action)
    critical invoke2
        studio->>+ds: ProviderRequest<br />(query_type + url-form-encoded fields)
        ds->tpr: Request
        ds->>ds: Serialize inner data model to Blob
        ds->>-studio: Blob response
    end
    par Save the state of the notebook
        studio->>api: Storage of Blob as notebook metadata
    and Create new cells for studio
        alt application/vnd.fiberplane.cells{+json,+msgpack}
            studio->>studio: decode and create cells
        else create_cells
            studio->>+ds: Create Cells<br />(original query + blob)
            ds->>ds: Deserialize inner data model from Blob
            ds->>-studio: Cell commands
        end
    end
    studio->>-user: Display cells
```

## Create a new provider
Adding a new provider can be done following those steps. This section is meant
to be used as a tutorial, sprinkled with explanations.

### Necessary specifications
Before going in, you need to know at least what is going to be the configuration
needed for the provider, as well as a starting set of queries you want to
support.

#### Configuration
The configuration of a provider must be deserializable from JSON and YAML
formats[^deserconf].

[^deserconf]: Deserialization is an important step to instantiate Data Sources
    for a Proxy instance. Proxy configuration is mostly a serialized form of
    configurations in `data_sources.yml`

The configuration can and must _contain credentials_[^plaintextcreds], as it is the
object used by the [Proxy](https://github.com/fiberplane/proxy) later to connect
to the third-party resource.

[^plaintextcreds]: Both the proxy instance and the `data_sources.yml` file used
    to configure it are under the total control of the customer, so it is okay
    to use plain text credentials there.

#### Request types
A provider may support multiple request types, so that each query will prompt
users for the correct arguments and the provider will use the correct API from
its third-party resource. Listing the requests we want to implement is important
to scope the provider.

For example, for a Prometheus provider we might want:
- a graph query that displays the result of a PromQL query in a graph, and
- a "list metrics" query that returns the complete list of metrics known to the
  Prometheus instance with their help text/description
  
There are [special request
types](../fiberplane/src/protocols/providers/constants.rs) that are used by
Fiberplane products to enhance the Studio experience, and supporting them would
be nice to have:
- `status` is used to return the status of the link between the data source and
  the third-party resource.
- `suggestions` is used to return context aware autocomplete suggestions.

### Proxy Provider implementation
You can use the code in any of the Providers as an inspiration to make your own
provider that follows the specifications made earlier. A few bullet point advices:

#### Naming
Pick a name without spaces for the provider, and it is going to be easier if you
only use lowercase characters and underscore to produce
names[^lowercaseproviders], such as:
- `prometheus`
- `https`
- `aws_cloudwatch`

[^lowercaseproviders]: using only lowercase chars and underscore means the name
    of the folder will match the name of the rust library/sub-crate associated
    and it will make referencing the provider more robust across products.

#### Configuration
- As the configuration will mostly be deserialized from YAML, it is advised to
  use `camelCasing` for the keys in the configuration object.
- Add tests for the deserialization of the configuration. It is way easier to
  see a unit test fail rather than having to debug a data source that cannot be
  instantiated inside a runtime later.
  
#### Inner representation of third-party resources
- Add new structs that represent the API of the third-party resource you are
  communicating with
  + the struct should have helper function to serialize to/deserialize from
    `Blob`s to centralize the conversion logic
  + the struct should have helper function that wrap the runtime provided calls
    (`make_http_request`) to return response of that new type
    
#### Creating cells as part of the response
There are 2 possible situations once the data source has the data from its
backing third-party resource.

If the **data is static** (that is, you already know the exact cells to be
created with their content from the third party resource), then you can directly:
- serialize the cells to JSON or msgpack format,
- return that content in a blob using the
  `application/vnd.fiberplane.cells+json` or
  `application/vnd.fiberplane.cells+msgpack` according to the chosen format
  
Once you return this, Studio will know how to unpack the data and create the
cells accordingly, saving an extra round-trip and decoding step.

If the **data is not static** (so the provider needs to run extra steps with the
data to create relevant cells in a notebook), then providers will be asked to
create the cells associated to the specific query through the bindings function
`create_cells` that is exported and implemented by the provider. The function
will receive the payload as a `Blob` wrapped in metadata information, the blob
being the same type as the one sent in the `ProviderResponse`.

### Making the Provider available to the Proxy
Since the Proxy automatically knows to scan a specific folder to look for
provider of a given type, making the new provider available is only a matter of
compiling the provider and putting it in the right place, which is handled by a
helper script.

#### Compiling the wasm provider for Proxy
For the time being, the providers are bundled in tree, meaning they need to be
generated. The generation is handled by the
[`update_providers.sh`](https://github.com/fiberplane/proxy/tree/main/scripts/update_providers.sh),
which needs to have its `PROVIDERS` variable modified to add the new
type of provider.

#### Registering the new provider type in Proxy
Add a data source configuration in the sample [yaml
configuration](https://github.com/fiberplane/proxy/tree/main/data_sources.yaml).
This configuration will be used if you run the Proxy locally, e.g. within a Tilt
setup.

### In-Studio Provider implementation
#### Compilation of the wasm provider for Studio

For the time being, the providers are bundled in tree, meaning they need to be
generated. The generation is handled by the
[`update_providers.sh`](https://github.com/fiberplane/studio/tree/main/tools/scripts/update_providers.sh),
which needs to have its `PROVIDERS` variable modified to add the new
integration.

#### Registering the new provider type in Studio
##### Bundling the wasm plugin in the app
The registration phase where Studio picks locally available wasm plugins happens
in
[`src/services/DataSources/registerProviders.ts`](https://github.com/fiberplane/studio/tree/main/src/services/DataSources/registerProviders.ts)

##### Allow direct access configuration modal to choose the new data source
> Note: this has been postponed for now. Having to recreate the modal is tough.
> The step is going to be better if tried at the very end of the integration,
> after testing a source configured through a Proxy

The "direct access" configuration modal has a drop-down menu to [choose the
provider
from](https://github.com/fiberplane/studio/blob/main/src/components/Modals/ConfigureDirectAccessModal/ConfigureDirectAccessModal.tsx#L140).
The new provider type must be added there as well.

#### Test of the provider locally in a notebook
Test the new provider in a notebook locally through a full tilt setup is a good
step to make sure nothing have been missed and that simple requests such as
status are working correctly.

##### Provision exemplar data source with credentials
Either a docker image (prometheus, loki...), or staging credentials for a SaaS
service (cloudwatch, cloud operations...) are going to be necessary to connect
the test provider to a proper third-party resource.

#### Usage of the new provider in Studio
##### Addition of a new Slash Command
Note that we need to use the correct `QUERY_TYPE` to pass to the provider when
dispatching the `activateProviderCell` call from `slashCommandActions`.

That is all that's needed, as long as the provider is correctly activated in the
notebook and responds correctly to the status part

## Ergonomics notes and possible improvements
### TODO Open questions
- Unclear that mime_types in SupportedQueryTypes are actually used as key in
  Cell::data_links. Are they??
### Provider Protocol types
- `SupportedQueryType` could use a builder pattern probably to look less verbose
  and error-prone; using something like a [derive
  builder](https://docs.rs/derive_builder/latest/derive_builder/) macro with
  custom setters.
- Same thing with `QueryField`
### Provider Protocol misc.
- There are a lot of places to synchronize, it would be better if the wasm
  plugin was able to declare at least part of its usage
### Provider developer experience
- Adding a macro that generates a `Blob -> Result<QuerySchema>` function from
  the `SupportedQueryType` structure would be nice. That would reduce the
  boilerplate in the `invoke2` handlers
- The `update_providers` script could probably use optional arguments to not
  update everything at once
### Studio integration
- Dynamically computing the list of providers from the providers registry might
  be a nice improvement (at least I was expecting it to work this way.)
- Using the result of `get_supported_query_type` to auto-fill
  `slashCommandActions` would be great too. For that to work best,
  `SupportedQueryType` in the protocol-specific types could use some kind of
  `Title` field which would be a "pretty", short, human-readable description of
  the query, as used today in the SlashCommands widget.
