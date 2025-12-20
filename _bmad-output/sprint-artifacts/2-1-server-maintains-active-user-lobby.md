# Story 2.1: Server Maintains Active User Lobby

**Epic:** 2 - Presence - Online Lobby & Real-Time Updates  
**Story ID:** 2.1  
**Status:** ready-for-dev  
**Priority:** High - Foundation for Epic 2  
**Estimated Complexity:** Medium  
**Dependencies:** Epic 1 completion (authentication system)

## 🎯 Story Overview

This story establishes the server-side foundation for presence tracking by maintaining an in-memory data structure that tracks all currently authenticated users. Without this critical infrastructure, the lobby cannot function, users cannot see who's available to message, and real-time presence updates are impossible.

**Business Value:** Enables the core presence infrastructure that allows users to see who's online and receive real-time updates when people connect or disconnect - essential for effective messaging functionality.

**Technical Value:** Creates the authoritative server state for presence tracking, providing the data foundation for all subsequent Epic 2 stories and enabling message routing based on online status.

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a server application,
I want to maintain an in-memory list of all currently authenticated users with their public keys,
so that I can inform clients who is available to message and route messages to online recipients.

## Acceptance Criteria

1. [Add acceptance criteria from epics/PRD]

## Tasks / Subtasks

- [ ] Task 1 (AC: #)
  - [ ] Subtask 1.1
- [ ] Task 2 (AC: #)
  - [ ] Subtask 2.1

## Dev Notes

- Relevant architecture patterns and constraints
- Source tree components to touch
- Testing standards summary

### Project Structure Notes

- Alignment with unified project structure (paths, modules, naming)
- Detected conflicts or variances (with rationale)

### References

- Cite all technical details with source paths and sections, e.g. [Source: docs/<file>.md#Section]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
