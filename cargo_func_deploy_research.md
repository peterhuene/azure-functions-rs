# Research for Adding cargo func deploy Command

## Context

Azure Function runtimes allow for a command-line based approach to publishing the locally run function app into Azure. It would be beneficial to mirror that approach with a tool leveraging `cargo`. Currently the `cargo func` command is used to create function apps and add functions to that app.  

## Current User Story: Deploying a Function App to Azure

### Requirements

- All requirements for building the Rust Function App must be met (for development purposes)
- Docker 18.06+ with BuildKit support  

### Deployment Story

(From README as of 22 March 2019)

Build the image using Docker

``` bash
docker build -t $TAG_NAME .
```

`$TAG_NAME` can be, for example, `username/my-functions-app`.

Use `docker push` to push the image to a repository accessible by Azure Functions.

``` bash
docker push $TAG_NAME
```

Create a Linux Azure Function App. Publish with "Docker Image".  
From the "Configure Container" section, select the repo and enter the image pushed.  

### Pure Command Line Implementation

We can leverage the Azure CLI to better programmatize this task. (Programmatizing (what a word...) on the path to automation)

Should a dependency on the CLI be used? What is the cost of adding this dependency? Are there RESTful ways of doing this that we can hook into and not add dependencies?  

The CLI can be downloaded [here](https://docs.microsoft.com/en-us/cli/azure/install-azure-cli?view=azure-cli-latest).  
(The CLI can also be run using a Docker container.)

Using the CLI, the commands can be issued:

Create a Resource Group. This is where your container registry and function app will be deployed.

``` bash
az group create --name resourceGroupName --location eastus
```

Create a Container Registry.  

``` bash
az acr create --resource-group resourceGroupName --name containerRegistryName --sku Basic
```

Log into the registry

``` bash
az acr login --name containerRegistryName
```

Build the image (with the tag including the acr name? I don't fully understand this part... Not great at Docker yet)

``` bash
docker build -t $TAG_NAME .
```

Push the image to the registry

``` bash
docker push $TAG_NAME
```

Create an Azure Storage Account

``` bash
az storage account create --name storageName --location westeurope --resource-group resourceGroupName
```

Create a Linux App Service Plan. Currently, Azure Functions running Docker images on Linux doesn't support consumption plans :(.

``` bash
az appservice plan create --name myAppServicePlan --resource-group resourceGroupName --sku B1 --is-linux
```

Deploy your image. (I need to test this... How do I make sure it is connected to the ACR? This is pulled from a reference that uses Docker Hub. Should we just use Docker Hub for this? What are the security implications?)

``` bash
az functionapp create --name <app_name> --storage-account  <storage_name>  --resource-group resourceGroupName \
--plan myAppServicePlan --deployment-container-image-name $TAG_NAME
```

Configure the function app

``` bash
storageConnectionString=$(az storage account show-connection-string \
--resource-group resourceGroupName --name <storage_name> \
--query connectionString --output tsv)

az functionapp config appsettings set --name <app_name> \
--resource-group resourceGroupName \
--settings AzureWebJobsDashboard=$storageConnectionString \
AzureWebJobsStorage=$storageConnectionString
```