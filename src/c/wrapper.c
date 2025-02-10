#include "json.h"
#include "stdio.h"
#include "stdlib.h"
#include "string.h"

const size_t BUFFER_CHUNK = 8192;

struct minalloc_buffer {
    unsigned char* ptr;
    size_t size;
    size_t index;
} typedef minalloc_buffer;

void minflush(minalloc_buffer buffer) {
    free(buffer.ptr);
    buffer.size = 0;
    buffer.index = 0;
}

void* minalloc(minalloc_buffer buffer, size_t size) {
    if (buffer.ptr == NULL || buffer.index + size > buffer.size) {
        size_t new_size = buffer.size + ((size > BUFFER_CHUNK) ? size : BUFFER_CHUNK);
        unsigned char* new_buffer = realloc(buffer.ptr, new_size);
        if (!new_buffer) {
            fprintf(stderr, "Memory allocation failed!\n");
            exit(EXIT_FAILURE);
        }
        buffer.ptr = new_buffer;
        buffer.size = new_size;
    }
    void* start = buffer.ptr + buffer.index;
    memset(start, 0, size);
    buffer.index += size;
    return start;
}

typedef struct {
    unsigned int id;
    char name[64];
    char bodyPart[16];
} Exercise;

const char* id_str = "id";
const char* name_str = "name";
const char* body_part_str = "body_part";

void add_exercise_details(minalloc_buffer buffer, Exercise exercise, struct json_object* object) {
    struct json_pair* body_part_ptr = minalloc(buffer, sizeof(struct json_pair));
    body_part_ptr->key = (char*)body_part_str;
    body_part_ptr->value.type = JSON_STRING;
    int size = strlen(exercise.bodyPart);
    char* b = minalloc(buffer, size+1);
    strncpy(b, exercise.bodyPart, size+1);
    body_part_ptr->value.value.string = b;
    body_part_ptr->next = NULL;

    struct json_pair* name_ptr = minalloc(buffer, sizeof(struct json_pair));
    name_ptr->key = (char*)name_str;
    name_ptr->value.type = JSON_STRING;
    size = strlen(exercise.name);
    b = minalloc(buffer, size+1);
    strncpy(b, exercise.name, size+1);
    name_ptr->value.value.string = b;
    name_ptr->next = body_part_ptr;

    struct json_pair* id_ptr = minalloc(buffer, sizeof(struct json_pair));
    id_ptr->key = (char*)id_str;
    id_ptr->value.value.integer = exercise.id;
    id_ptr->value.type = JSON_INT;
    id_ptr->next = name_ptr;

    object->head = id_ptr;
    object->tail = body_part_ptr;
}


void exercises_to_json(Exercise* exercises, size_t num, char* out_buffer) {
    minalloc_buffer buffer = {
        .ptr = NULL,
        .size = 0,
        .index = 0
    };

    struct json_value exercises_array;
    exercises_array.type = JSON_ARRAY;
    struct json_array exercises_array_back;
    exercises_array_back.size = num;

    struct json_value* exercise_values = minalloc(buffer, num * sizeof(struct json_value));
    exercises_array_back.values = exercise_values;
    exercises_array.value.array = &exercises_array_back;
    struct json_object* exercise_values_objects = minalloc(buffer, num * sizeof(struct json_object));

    for (size_t i = 0; i < num; i++) {
        exercise_values[i].type = JSON_OBJECT;
        exercise_values[i].value.object = exercise_values_objects + i;
        add_exercise_details(buffer, exercises[i], exercise_values[i].value.object);
    }
    char* out = json_value_to_string(exercises_array);
    if (out != NULL) {
        snprintf(out_buffer, 8192, "%s", out);
        free(out);
    } else {
        snprintf(out_buffer, 8192, "Error generating JSON");
    }


    minflush(buffer);
}