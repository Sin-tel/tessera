--[[
todo 
crossplatform dlls
better ffi loading (see protoplug)
message parsing
]]

local ffi = require("ffi")

local lib = ffi.load(love.filesystem.getSource() .. "/lib/RtMidi")

ffi.cdef([[

//! \brief Wraps an RtMidi object for C function return statuses.
struct RtMidiWrapper {
        //! The wrapped RtMidi object.
        void* ptr;
        void* data;

        //! True when the last function call was OK.
        bool  ok;

        //! If an error occured (ok != true), set to an error message.
        const char* msg;
};

//! \brief Typedef for a generic RtMidi pointer.
typedef struct RtMidiWrapper* RtMidiPtr;

//! \brief Typedef for a generic RtMidiIn pointer.
typedef struct RtMidiWrapper* RtMidiInPtr;

//! \brief Typedef for a generic RtMidiOut pointer.
typedef struct RtMidiWrapper* RtMidiOutPtr;

//! \brief MIDI API specifier arguments.  See \ref RtMidi::Api.
enum RtMidiApi {
        RTMIDI_API_UNSPECIFIED,    /*!< Search for a working compiled API. */
        RTMIDI_API_MACOSX_CORE,    /*!< Macintosh OS-X CoreMIDI API. */
        RTMIDI_API_LINUX_ALSA,     /*!< The Advanced Linux Sound Architecture API. */
        RTMIDI_API_UNIX_JACK,      /*!< The Jack Low-Latency MIDI Server API. */
        RTMIDI_API_WINDOWS_MM,     /*!< The Microsoft Multimedia MIDI API. */
        RTMIDI_API_RTMIDI_DUMMY,   /*!< A compilable but non-functional API. */
        RTMIDI_API_NUM             /*!< Number of values in this enum. */
};

//! \brief Defined RtMidiError types. See \ref RtMidiError::Type.
enum RtMidiErrorType {
    RTMIDI_ERROR_WARNING,           /*!< A non-critical error. */
    RTMIDI_ERROR_DEBUG_WARNING,     /*!< A non-critical error which might be useful for debugging. */
    RTMIDI_ERROR_UNSPECIFIED,       /*!< The default, unspecified error type. */
    RTMIDI_ERROR_NO_DEVICES_FOUND,  /*!< No devices found on system. */
    RTMIDI_ERROR_INVALID_DEVICE,    /*!< An invalid device ID was specified. */
    RTMIDI_ERROR_MEMORY_ERROR,      /*!< An error occured during memory allocation. */
    RTMIDI_ERROR_INVALID_PARAMETER, /*!< An invalid parameter was specified to a function. */
    RTMIDI_ERROR_INVALID_USE,       /*!< The function was called incorrectly. */
    RTMIDI_ERROR_DRIVER_ERROR,      /*!< A system driver error occured. */
    RTMIDI_ERROR_SYSTEM_ERROR,      /*!< A system error occured. */
    RTMIDI_ERROR_THREAD_ERROR       /*!< A thread error occured. */
};

/*! \brief The type of a RtMidi callback function.
 *
 * \param timeStamp   The time at which the message has been received.
 * \param message     The midi message.
 * \param userData    Additional user data for the callback.
 *
 * See \ref RtMidiIn::RtMidiCallback.
 */
typedef void(* RtMidiCCallback) (double timeStamp, const unsigned char* message,
                                                                 size_t messageSize, void *userData);


/* RtMidi API */

/*! \brief Determine the available compiled MIDI APIs.
 *
 * If the given `apis` parameter is null, returns the number of available APIs.
 * Otherwise, fill the given apis array with the RtMidi::Api values.
 *
 * \param apis  An array or a null value.
 * \param apis_size  Number of elements pointed to by apis
 * \return number of items needed for apis array if apis==NULL, or
 *         number of items written to apis array otherwise.  A negative
 *         return value indicates an error.
 *
 * See \ref RtMidi::getCompiledApi().
*/
 int rtmidi_get_compiled_api (enum RtMidiApi *apis, unsigned int apis_size);

//! \brief Return the name of a specified compiled MIDI API.
//! See \ref RtMidi::getApiName().
 const char *rtmidi_api_name(enum RtMidiApi api);

//! \brief Return the display name of a specified compiled MIDI API.
//! See \ref RtMidi::getApiDisplayName().
 const char *rtmidi_api_display_name(enum RtMidiApi api);

//! \brief Return the compiled MIDI API having the given name.
//! See \ref RtMidi::getCompiledApiByName().
 enum RtMidiApi rtmidi_compiled_api_by_name(const char *name);

//! \internal Report an error.
 void rtmidi_error (enum RtMidiErrorType type, const char* errorString);

/*! \brief Open a MIDI port.
 *
 * \param port      Must be greater than 0
 * \param portName  Name for the application port.
 *
 * See RtMidi::openPort().
 */
 void rtmidi_open_port (RtMidiPtr device, unsigned int portNumber, const char *portName);

/*! \brief Creates a virtual MIDI port to which other software applications can
 * connect.
 *
 * \param portName  Name for the application port.
 *
 * See RtMidi::openVirtualPort().
 */
 void rtmidi_open_virtual_port (RtMidiPtr device, const char *portName);

/*! \brief Close a MIDI connection.
 * See RtMidi::closePort().
 */
 void rtmidi_close_port (RtMidiPtr device);

/*! \brief Return the number of available MIDI ports.
 * See RtMidi::getPortCount().
 */
 unsigned int rtmidi_get_port_count (RtMidiPtr device);

/*! \brief Return a string identifier for the specified MIDI input port number.
 * See RtMidi::getPortName().
 */
 const char* rtmidi_get_port_name (RtMidiPtr device, unsigned int portNumber);

/* RtMidiIn API */

//! \brief Create a default RtMidiInPtr value, with no initialization.
 RtMidiInPtr rtmidi_in_create_default (void);

/*! \brief Create a  RtMidiInPtr value, with given api, clientName and queueSizeLimit.
 *
 *  \param api            An optional API id can be specified.
 *  \param clientName     An optional client name can be specified. This
 *                        will be used to group the ports that are created
 *                        by the application.
 *  \param queueSizeLimit An optional size of the MIDI input queue can be
 *                        specified.
 *
 * See RtMidiIn::RtMidiIn().
 */
 RtMidiInPtr rtmidi_in_create (enum RtMidiApi api, const char *clientName, unsigned int queueSizeLimit);

//! \brief Free the given RtMidiInPtr.
 void rtmidi_in_free (RtMidiInPtr device);

//! \brief Returns the MIDI API specifier for the given instance of RtMidiIn.
//! See \ref RtMidiIn::getCurrentApi().
 enum RtMidiApi rtmidi_in_get_current_api (RtMidiPtr device);

//! \brief Set a callback function to be invoked for incoming MIDI messages.
//! See \ref RtMidiIn::setCallback().
 void rtmidi_in_set_callback (RtMidiInPtr device, RtMidiCCallback callback, void *userData);

//! \brief Cancel use of the current callback function (if one exists).
//! See \ref RtMidiIn::cancelCallback().
 void rtmidi_in_cancel_callback (RtMidiInPtr device);

//! \brief Specify whether certain MIDI message types should be queued or ignored during input.
//! See \ref RtMidiIn::ignoreTypes().
 void rtmidi_in_ignore_types (RtMidiInPtr device, bool midiSysex, bool midiTime, bool midiSense);

/*! Fill the user-provided array with the data bytes for the next available
 * MIDI message in the input queue and return the event delta-time in seconds.
 *
 * \param message   Must point to a char* that is already allocated.
 *                  SYSEX messages maximum size being 1024, a statically
 *                  allocated array could
 *                  be sufficient.
 * \param size      Is used to return the size of the message obtained.
 *                  Must be set to the size of \ref message when calling.
 *
 * See RtMidiIn::getMessage().
 */
 double rtmidi_in_get_message (RtMidiInPtr device, unsigned char *message, size_t *size);

/* RtMidiOut API */

//! \brief Create a default RtMidiInPtr value, with no initialization.
 RtMidiOutPtr rtmidi_out_create_default (void);

/*! \brief Create a RtMidiOutPtr value, with given and clientName.
 *
 *  \param api            An optional API id can be specified.
 *  \param clientName     An optional client name can be specified. This
 *                        will be used to group the ports that are created
 *                        by the application.
 *
 * See RtMidiOut::RtMidiOut().
 */
 RtMidiOutPtr rtmidi_out_create (enum RtMidiApi api, const char *clientName);

//! \brief Free the given RtMidiOutPtr.
 void rtmidi_out_free (RtMidiOutPtr device);

//! \brief Returns the MIDI API specifier for the given instance of RtMidiOut.
//! See \ref RtMidiOut::getCurrentApi().
 enum RtMidiApi rtmidi_out_get_current_api (RtMidiPtr device);

//! \brief Immediately send a single message out an open MIDI output port.
//! See \ref RtMidiOut::sendMessage().
 int rtmidi_out_send_message (RtMidiOutPtr device, const unsigned char *message, int length);
]])

ffi.cdef([[
typedef struct
{
    double time;
    size_t size[1];
    unsigned char data[?];
} MidiEvent;
]])

local M = { C = lib }

local maxSize = 1024 --max sysex size
local msg = ffi.new("MidiEvent", maxSize)

local function toBits(num, bits)
    -- returns a table of bits, most significant first.
    bits = bits or math.max(1, select(2, math.frexp(num)))
    local t = {} -- will contain the bits
    for b = bits, 1, -1 do
        t[b] = math.fmod(num, 2)
        num = math.floor((num - t[b]) / 2)
    end
    return table.concat(t)
end

function M.createIn()
    local dev = lib.rtmidi_in_create_default()
    return ffi.gc(dev, lib.rtmidi_in_free)
end

function M.createOut()
    local dev = lib.rtmidi_out_create_default()
    return ffi.gc(dev, lib.rtmidi_out_free)
end

function M.ignoreTypes(device, midiSysex, midiTime, midiSense)
    lib.rtmidi_in_ignore_types(device, midiSysex, midiTime, midiSense)
end

function M.printPorts(device)
    local nPorts = lib.rtmidi_get_port_count(device)

    for i = 0, nPorts - 1 do
        local portName = ffi.string(lib.rtmidi_get_port_name(device, i))
        print(i, portName)
    end
end

function M.findPort(device, name)
    local nPorts = lib.rtmidi_get_port_count(device)

    for i = 0, nPorts - 1 do
        local portName = ffi.string(lib.rtmidi_get_port_name(device, i))
        print(string.lower(portName), string.lower(name))
        if string.match(string.lower(portName), string.lower(name)) then
            return i
        end
    end

    print('Midi port "' .. name .. '" not found!')
    return false
end

function M.openPort(device, p)
    local portName = lib.rtmidi_get_port_name(device, p)
    print("opening port: " .. ffi.string(portName))
    lib.rtmidi_open_port(device, p, portName)
end

function M.closePort(device)
    lib.rtmidi_close_port(device)
end

function M.newMessage(t)
    local new = ffi.new("MidiEvent", #t, { size = { #t }, data = t })
    return new
end

function M.sendMessage(device, msg)
    --print(msg.data[0], msg.size[0])
    lib.rtmidi_out_send_message(device, msg.data, msg.size[0])
end

function M.getMessage(device)
    msg.size[0] = ffi.cast("size_t", maxSize)
    msg.time = lib.rtmidi_in_get_message(device, msg.data, msg.size)
    return msg, tonumber(msg.size[0])
end

function M.isSysex(m)
    return m.data[0] == 0xf0 and m.data[1] == 0x00
end

return M
