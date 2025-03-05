#include "json.h"
#include "stdio.h"
#include "stdlib.h"
#include "string.h"
#include "stdint.h"
#include "stddef.h"

const size_t BUFFER_CHUNK = 8192;

struct minalloc_buffer {
    unsigned char* ptr;
    size_t size;
    size_t index;
} typedef minalloc_buffer;

//TODO: Sort this out properly
void* minalloc(minalloc_buffer *buffer, size_t size) {
    if (buffer->ptr == NULL || buffer->index + size > buffer->size) {
        size_t new_size = buffer->size + ((size > BUFFER_CHUNK) ? size : BUFFER_CHUNK);
        unsigned char* new_ptr = realloc(buffer->ptr, new_size);
        if (!new_ptr) {
            fprintf(stderr, "Memory allocation failed!\n");
            exit(EXIT_FAILURE);
        }
        buffer->ptr = new_ptr;
        buffer->size = new_size;
    }

    void* start = buffer->ptr + buffer->index;
    memset(start, 0, size);
    buffer->index += size;
    return start;
}

void minflush(minalloc_buffer *buffer) {
    if (buffer->ptr) {
        free(buffer->ptr);
        buffer->ptr = NULL;
    }
    buffer->size = 0;
    buffer->index = 0;
}


typedef struct {
    unsigned int id;
    char name[64];
    char bodyPart[16];
} Exercise;

const char* id_str = "id";
const char* name_str = "name";
const char* body_part_str = "body_part";

void add_exercise_details(minalloc_buffer* buffer, Exercise exercise, struct json_object* object) {
    struct json_pair* body_part_ptr = minalloc(buffer, sizeof(struct json_pair));
    body_part_ptr->key = (char*)body_part_str;
    body_part_ptr->value.type = JSON_STRING;
    int size = strlen(exercise.bodyPart);
    char* b = minalloc(buffer, size+1);
    strncpy(b, exercise.bodyPart, size+1);
    body_part_ptr->value.as.string = b;
    body_part_ptr->next = NULL;

    struct json_pair* name_ptr = minalloc(buffer, sizeof(struct json_pair));
    name_ptr->key = (char*)name_str;
    name_ptr->value.type = JSON_STRING;
    size = strlen(exercise.name);
    b = minalloc(buffer, size+1);
    strncpy(b, exercise.name, size+1);
    name_ptr->value.as.string = b;
    name_ptr->next = body_part_ptr;

    struct json_pair* id_ptr = minalloc(buffer, sizeof(struct json_pair));
    id_ptr->key = (char*)id_str;
    id_ptr->value.as.integer = exercise.id;
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

    struct json_value* exercise_values = minalloc(&buffer, num * sizeof(struct json_value));
    exercises_array_back.values = exercise_values;
    exercises_array.as.array = &exercises_array_back;
    struct json_object* exercise_values_objects = minalloc(&buffer, num * sizeof(struct json_object));

    for (size_t i = 0; i < num; i++) {
        exercise_values[i].type = JSON_OBJECT;
        exercise_values[i].as.object = exercise_values_objects + i;
        add_exercise_details(&buffer, exercises[i], exercise_values[i].as.object);
    }
    char* out = json_value_to_string(exercises_array);
    if (out != NULL) {
        snprintf(out_buffer, 8192, "%s", out);
        free(out);
    } else {
        snprintf(out_buffer, 8192, "Error generating JSON");
    }


    minflush(&buffer);
}

struct Set{
    uint32_t workout_exercise_id;
    uint32_t set_number;
    double weight;
    uint32_t reps;
}typedef Set;  

struct Sets {
    Set* content;
    uint32_t count;
} typedef Sets; 


_Thread_local minalloc_buffer local_buffer = {
    .ptr = NULL,
    .size = 0,
    .index = 0
};

void* local_alloc(size_t size) {
    return minalloc(&local_buffer, size);
}

struct json_pair* get_pair(struct json_object* object, const char* key) {
    struct json_pair* got = NULL;

    struct json_pair* head = object->head;
    while (head != NULL)
    {
        if(strncmp(head->key, key, strlen(key)) == 0) {
            got = head;
        }
        head = head->next;
    }

    return got;
}

void append_sets(Sets* sets, uint32_t exercise_id, struct json_value json_set, uint32_t index) {

    if(json_set.type != JSON_OBJECT) {
        printf("Failed: 5\n");
        return;
    }

    struct json_object* object = json_set.as.object;

    struct json_pair* reps_pair = get_pair(object, "reps");
    struct json_pair* weight_pair = get_pair(object, "weight");

    if(reps_pair == NULL || weight_pair == NULL) {
        printf("Failed: 6\n");
        return;
    }

    if(reps_pair->value.type != JSON_INT || !(weight_pair->value.type == JSON_INT || weight_pair->value.type == JSON_DECIMAL)) {
        printf("Failed: 7\n");
        return;
    }

    uint32_t reps = reps_pair->value.as.integer;
    double weight = 0;

    if(weight_pair->value.type == JSON_INT) {
        weight = (double)weight_pair->value.as.integer;
    } else {
        weight = weight_pair->value.as.decimal;
    }

    
    sets->content[sets->count].workout_exercise_id = exercise_id;
    sets->content[sets->count].set_number = index;
    sets->content[sets->count].weight = weight;
    sets->content[sets->count].reps = reps;

    sets->count++;

    return;
}

void append_exercise(Sets* sets, struct json_value exercise) {

    if(exercise.type != JSON_OBJECT) {
        return;
    }

    struct json_object* object = exercise.as.object;

    struct json_pair* exercise_id_pair = get_pair(object, "exercise_id");
    struct json_pair* sets_pair = get_pair(object, "sets");

    if(exercise_id_pair == NULL || sets_pair == NULL) {
        printf("Failed: 3\n");
        return;
    } 

    if(exercise_id_pair->value.type != JSON_INT || sets_pair->value.type != JSON_ARRAY) {
        printf("Failed: 4\n");
        return;
    }

    uint32_t exercise_id = exercise_id_pair->value.as.integer;
    struct json_array* sets_array = sets_pair->value.as.array;

    printf("Sets in exercise: %ld\n", sets_array->size);

    for (uint32_t i = 0; i < sets_array->size; i++) {
        append_sets(sets, exercise_id, sets_array->values[i], i);
    }
    
    return;
}

const size_t MAX_SETS = 128;

Sets json_to_exercise(const char* json_string, uint32_t user_id) {

    static const Sets FAILED_SETS = { .content = NULL, .count = 0 };

    uint32_t count = 0;

    struct json_value parsed_value = json_parse_string(json_string, local_alloc);

    if (parsed_value.type != JSON_OBJECT) {

        if(parsed_value.type == JSON_ERROR) {
            printf("Got: <%s>\n", json_string);
            printf("Error: <%s>\n", parsed_value.as.string);
        }

        minflush(&local_buffer);
        printf("Failed: 1\n");
        return FAILED_SETS;
    }

    struct json_object* object = parsed_value.as.object;

    struct json_pair* user_id_pair = get_pair(object, "user_id");
    struct json_pair* exercises_pair = get_pair(object, "exercises");    

    if(user_id_pair == NULL || exercises_pair == NULL) {
        minflush(&local_buffer);
        printf("Failed: 2\n");
        return FAILED_SETS;
    }

    char* user_id_str = json_value_to_string(user_id_pair->value);
    char* exercises_str = json_value_to_string(exercises_pair->value);

    printf("User ID = %s\nExercises: %s\n", user_id_str, exercises_str);
    free(user_id_str);
    free(exercises_str);

    if(exercises_pair->value.type != JSON_ARRAY) {
        minflush(&local_buffer);
        return FAILED_SETS;
    }

    struct json_array* exercises = exercises_pair->value.as.array;

    Set* content = malloc(sizeof(Set)*MAX_SETS);
    memset(content, 0, (sizeof(Set)*MAX_SETS));

    Sets sets = { .content = content, .count = count };

    for (size_t i = 0; i < exercises->size; i++)
    {
        append_exercise(&sets, exercises->values[i]);
    }
    
    printf("No. Sets = %d\n", sets.count);


    minflush(&local_buffer);
    return sets;
}

void free_sets(Sets sets) {
    free(sets.content);
}