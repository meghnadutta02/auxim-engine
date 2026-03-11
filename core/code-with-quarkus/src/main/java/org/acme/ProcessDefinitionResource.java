package org.acme;

import jakarta.transaction.Transactional;
import jakarta.ws.rs.*;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import java.util.UUID;

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
}