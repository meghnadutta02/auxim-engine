package org.acme;

import io.quarkus.hibernate.orm.panache.PanacheEntityBase;
import jakarta.persistence.*;
import org.hibernate.annotations.JdbcTypeCode;
import org.hibernate.type.SqlTypes;
import com.fasterxml.jackson.databind.JsonNode;
import java.util.UUID;
import java.util.List;

@Entity
@Table(name = "process_instance")
public class ProcessInstance extends PanacheEntityBase {
    
    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    public UUID id;

    public String definitionId; // Link back to definition
    public String status; // "RUNNING", "WAITING", "COMPLETED"

    @JdbcTypeCode(SqlTypes.JSON)
    @Column(columnDefinition = "jsonb")
    public JsonNode currentState; // The payload data

    @JdbcTypeCode(SqlTypes.JSON)
    @Column(columnDefinition = "jsonb")
    public List<String> currentNodeIds; // Where the engine is currently paused
}