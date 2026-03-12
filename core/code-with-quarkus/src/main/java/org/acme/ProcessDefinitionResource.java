package org.acme;

import jakarta.transaction.Transactional;
import jakarta.ws.rs.*;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import java.util.UUID;
import java.util.List;

@Path("/process-definitions")
@Produces(MediaType.APPLICATION_JSON)
@Consumes(MediaType.APPLICATION_JSON)
public class ProcessDefinitionResource {

    @POST
    @Transactional
    public Response createDefinition(ProcessDefinition def) {
        // Hurdle-04 Validator logic: Ensure there's a START node
        boolean hasStart = false;
        if (def.graphData != null && def.graphData.has("nodes")) {
            for (var node : def.graphData.get("nodes")) {
                if ("START".equals(node.get("nodeType").asText())) {
                    hasStart = true;
                    break;
                }
            }
        }

        if (!hasStart) {
            return Response.status(400).entity("{\"error\": \"Invalid DSL: Must contain a START node\"}").build();
        }

        def.version = 1;
        def.persist();
        return Response.status(201).entity(def).build();
    }

    @GET
    @Path("/{id}")
    public ProcessDefinition getDefinition(@PathParam("id") UUID id) {
        return ProcessDefinition.findById(id);
    }

    @GET
    public Response listDefinitions(
            @QueryParam("page") @DefaultValue("0") int page,
            @QueryParam("size") @DefaultValue("10") int size) {

        List<ProcessDefinition> definitions = ProcessDefinition.findAll().page(page, size).list();
        return Response.ok(definitions).build();
    }

    @PUT
    @Path("/{id}")
    @Transactional
    public Response updateDefinition(@PathParam("id") UUID id, ProcessDefinition updatedDef) {
        ProcessDefinition existing = ProcessDefinition.findById(id);
        if (existing == null) {
            return Response.status(404).entity("{\"error\": \"Definition not found\"}").build();
        }

        boolean hasStart = false;
        if (updatedDef.graphData != null && updatedDef.graphData.has("nodes")) {
            for (var node : updatedDef.graphData.get("nodes")) {
                if ("START".equals(node.get("nodeType").asText())) {
                    hasStart = true;
                    break;
                }
            }
        }

        if (!hasStart) {
            return Response.status(400).entity("{\"error\": \"Invalid DSL: Update must contain a START node\"}")
                    .build();
        }

        existing.status = updatedDef.status;
        existing.graphData = updatedDef.graphData;
        existing.version = existing.version + 1;

        return Response.ok(existing).build();
    }
}