package org.acme;

import jakarta.transaction.Transactional;
import jakarta.ws.rs.*;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import java.util.*;
import org.eclipse.microprofile.rest.client.inject.RestClient;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

@Path("/executions")
@Produces(MediaType.APPLICATION_JSON)
@Consumes(MediaType.APPLICATION_JSON)
public class ExecutionResource {

    @RestClient
    RestateClient restateClient;

    public static class StartRequest {
        public String processDefinitionId;
        public JsonNode input;
    }

    @POST
    @Transactional
    public Response startExecution(StartRequest req) {
        // 1. Verify Definition Exists
        ProcessDefinition def = ProcessDefinition.findById(UUID.fromString(req.processDefinitionId));
        if (def == null || !"PUBLISHED".equals(def.status)) {
            return Response.status(400).entity("{\"error\": \"Definition not found or not published\"}").build();
        }

        // 2. Initialize Process Instance State
        ProcessInstance instance = new ProcessInstance();
        instance.definitionId = def.id.toString();
        instance.status = "RUNNING";
        instance.currentState = req.input.deepCopy();

        // Find the START node ID from the graph
        String startNodeId = def.graphData.get("nodes").get(0).get("id").asText();
        instance.currentNodeIds = List.of(startNodeId);

        instance.persist();

        // 3. Trigger the Rust Engine via Restate (FIXED PAYLOAD)
        Map<String, Object> restatePayload = new HashMap<>();
        restatePayload.put("executionKey", instance.id.toString());

        // We must send the actual graph arrays to Rust!
        restatePayload.put("nodes", def.graphData.get("nodes"));

        // Handle edges carefully in case it's missing/null in the DB
        JsonNode edges = def.graphData.get("edges");
        restatePayload.put("edges", edges != null ? edges : new ObjectMapper().createArrayNode());

        restatePayload.put("input", req.input);

        restateClient.triggerRustEngine(restatePayload);

        return Response.status(201).entity(instance).build();
    }

    @GET
    @Path("/{id}")
    public ProcessInstance getExecution(@PathParam("id") UUID id) {
        return ProcessInstance.findById(id);
    }

    @GET
    public Response listExecutions(
            @QueryParam("page") @DefaultValue("0") int page,
            @QueryParam("size") @DefaultValue("10") int size) {

        List<ProcessInstance> instances = ProcessInstance.findAll().page(page, size).list();
        return Response.ok(instances).build();
    }
}