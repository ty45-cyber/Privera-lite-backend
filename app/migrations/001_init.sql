CREATE TABLE IF NOT EXISTS users (
    id           VARCHAR(36)  PRIMARY KEY,
    email        VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name    VARCHAR(255) NOT NULL,
    role         VARCHAR(50)  NOT NULL DEFAULT 'employee',
    is_active    BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at   DATETIME     NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS payroll_batches (
    id            VARCHAR(36)  PRIMARY KEY,
    name          VARCHAR(255) NOT NULL,
    period_start  VARCHAR(20)  NOT NULL,
    period_end    VARCHAR(20)  NOT NULL,
    total_records INT          NOT NULL DEFAULT 0,
    status        VARCHAR(50)  NOT NULL DEFAULT 'draft',
    created_by    VARCHAR(36)  NOT NULL,
    created_at    DATETIME     NOT NULL,
    INDEX idx_payroll_batches_created_by (created_by),
    INDEX idx_payroll_batches_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS payroll_records (
    id                   VARCHAR(36)   PRIMARY KEY,
    batch_id             VARCHAR(36)   NOT NULL,
    employee_id          VARCHAR(100)  NOT NULL,
    employee_name        VARCHAR(255)  NOT NULL,
    department           VARCHAR(100)  NOT NULL,
    encrypted_salary     TEXT          NOT NULL,
    encrypted_deductions TEXT          NOT NULL,
    encrypted_net        TEXT          NOT NULL,
    currency             VARCHAR(10)   NOT NULL DEFAULT 'USD',
    INDEX idx_payroll_records_batch (batch_id),
    FOREIGN KEY (batch_id) REFERENCES payroll_batches(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS treasury_requests (
    id                 VARCHAR(36)    PRIMARY KEY,
    title              VARCHAR(255)   NOT NULL,
    amount             DECIMAL(18,2)  NOT NULL,
    currency           VARCHAR(10)    NOT NULL DEFAULT 'USD',
    purpose            TEXT           NOT NULL,
    risk_level         VARCHAR(20)    NOT NULL DEFAULT 'pending',
    status             VARCHAR(50)    NOT NULL DEFAULT 'pending',
    required_approvals INT            NOT NULL DEFAULT 2,
    current_approvals  INT            NOT NULL DEFAULT 0,
    requested_by       VARCHAR(36)    NOT NULL,
    created_at         DATETIME       NOT NULL,
    updated_at         DATETIME       NOT NULL,
    INDEX idx_treasury_status (status),
    INDEX idx_treasury_requested_by (requested_by)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS treasury_approvals (
    id           VARCHAR(36)  PRIMARY KEY,
    request_id   VARCHAR(36)  NOT NULL,
    approver_id  VARCHAR(36)  NOT NULL,
    approver_name VARCHAR(255) NOT NULL,
    decision     VARCHAR(20)  NOT NULL,
    note         TEXT,
    decided_at   DATETIME     NOT NULL,
    UNIQUE KEY uq_approver_request (request_id, approver_id),
    FOREIGN KEY (request_id) REFERENCES treasury_requests(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS proposals (
    id              VARCHAR(36)  PRIMARY KEY,
    title           VARCHAR(255) NOT NULL,
    description     TEXT         NOT NULL,
    status          VARCHAR(50)  NOT NULL DEFAULT 'active',
    created_by      VARCHAR(36)  NOT NULL,
    voting_ends_at  DATETIME     NOT NULL,
    created_at      DATETIME     NOT NULL,
    INDEX idx_proposals_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS votes (
    id             VARCHAR(36) PRIMARY KEY,
    proposal_id    VARCHAR(36) NOT NULL,
    voter_id       VARCHAR(36) NOT NULL,
    encrypted_vote TEXT        NOT NULL,
    cast_at        DATETIME    NOT NULL,
    UNIQUE KEY uq_voter_proposal (proposal_id, voter_id),
    FOREIGN KEY (proposal_id) REFERENCES proposals(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS audit_requests (
    id            VARCHAR(36)  PRIMARY KEY,
    auditor_id    VARCHAR(36)  NOT NULL,
    auditor_name  VARCHAR(255) NOT NULL,
    resource_type VARCHAR(50)  NOT NULL,
    resource_id   VARCHAR(36)  NOT NULL,
    reason        TEXT         NOT NULL,
    status        VARCHAR(20)  NOT NULL DEFAULT 'pending',
    created_at    DATETIME     NOT NULL,
    INDEX idx_audit_requests_status (status),
    INDEX idx_audit_requests_auditor (auditor_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS audit_logs (
    id            VARCHAR(36)  PRIMARY KEY,
    actor_id      VARCHAR(36)  NOT NULL,
    actor_role    VARCHAR(50)  NOT NULL,
    action        VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50)  NOT NULL,
    resource_id   VARCHAR(36)  NOT NULL,
    metadata      TEXT,
    occurred_at   DATETIME     NOT NULL,
    INDEX idx_audit_logs_actor (actor_id),
    INDEX idx_audit_logs_resource (resource_type, resource_id),
    INDEX idx_audit_logs_occurred (occurred_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;