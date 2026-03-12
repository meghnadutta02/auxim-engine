package org.acme;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.core.MediaType;
import org.eclipse.microprofile.rest.client.inject.RegisterRestClient;

import com.fasterxml.jackson.databind.JsonNode;

@RegisterRestClient(configKey = "restate-api")
public interface RestateClient {

    @Path("/TriggerService/on_dsl_start")
    @POST
    @Consumes(MediaType.APPLICATION_JSON)
    void triggerRustEngine(Object requestBody);

    @POST
    @Path("/DslWorkflowRunner/{executionId}/complete_user_task")
    @Consumes(MediaType.APPLICATION_JSON)
    void completeUserTask(@PathParam("executionId") String executionId, JsonNode userInput);
}